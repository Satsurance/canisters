use candid::{Nat, Principal};
use sha2::{Sha256, Digest};
use std::cell::RefCell;
use ic_cdk::api::call::call;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, StableCell};
use serde_json;

pub mod types;
// Re-export for external use
pub use types::{Account, Deposit, DepositError, TransferArg, TransferError};

type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static DEPOSITS: RefCell<StableBTreeMap<u64, String, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
        )
    );

    static TOKEN_ID: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))),
            Principal::anonymous()
        ).expect("Failed to initialize StableCell")
    );

    static DEPOSIT_COUNTER: RefCell<StableCell<u64, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2))),
            0u64
        ).expect("Failed to initialize StableCell")
    );
}

#[ic_cdk::query]
pub fn get_deposit_subaccount(user: Principal, timelock: u64) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(user.as_slice());
    hasher.update(timelock.to_be_bytes());
    hasher.finalize().into()
}

#[ic_cdk::init]
pub fn init(token_id: Option<Principal>) {
    if let Some(id) = token_id {
        TOKEN_ID.with(|cell| {
            cell.borrow_mut().set(id).ok();
        });
    }
}

#[ic_cdk::update]
pub async fn deposit(user: Principal, timelock: u64) -> Result<(), types::DepositError> {
    if timelock == 0 {
        return Err(types::DepositError::InvalidTimelock);
    }
    
    let ledger_principal = TOKEN_ID.with(|cell| {
        let stored = cell.borrow().get().clone();
        if stored == Principal::anonymous() {
            None
        } else {
            Some(stored)
        }
    }).ok_or(types::DepositError::LedgerNotSet)?;

    // Generate subaccount
    let subaccount = get_deposit_subaccount(user, timelock);
    let from_account = types::Account {
        owner: ic_cdk::api::id(),
        subaccount: Some(subaccount.to_vec()),
    };

    // Check balance in subaccount
    let balance_result: Result<(Nat,), _> = call(
        ledger_principal,
        "icrc1_balance_of",
        (from_account.clone(),)
    ).await;

    let balance = match balance_result {
        Ok((balance,)) => balance,
        Err(_) => return Err(types::DepositError::LedgerCallFailed),
    };

    let balance_u64: u64 = balance.0.clone().try_into().map_err(|_| types::DepositError::InternalError)?;

    if balance_u64 == 0 {
        return Err(types::DepositError::InsufficientBalance);
    }

    // Calculate transfer amount (balance minus fee)
    // First, let's get the transfer fee from the ledger
    let fee_result: Result<(Nat,), _> = call(
        ledger_principal,
        "icrc1_fee",
        ()
    ).await;
    
    let transfer_fee = match fee_result {
        Ok((fee,)) => fee,
        Err(_) => Nat::from(10_000u64), // Default fee if call fails
    };
    
    // Calculate amount to transfer (balance minus fee)
    let transfer_amount = if balance.0 > transfer_fee.0 {
        Nat::from(&balance.0 - &transfer_fee.0)
    } else {
        return Err(types::DepositError::InsufficientBalance); // Not enough for fee
    };

    // Transfer tokens from subaccount to canister's main account
    let transfer_args = (
        types::TransferArg {
            from_subaccount: Some(subaccount.to_vec()),
            to: types::Account {
                owner: ic_cdk::api::id(),
                subaccount: None, // Main account
            },
            amount: transfer_amount.clone(),
            fee: Some(transfer_fee),
            memo: Some(b"deposit_transfer".to_vec()),
            created_at_time: None,
        },
    );

    let transfer_result: Result<(Result<Nat, types::TransferError>,), _> = call(
        ledger_principal,
        "icrc1_transfer",
        transfer_args
    ).await;

    match transfer_result {
        Ok((Ok(_block_index),)) => {
            // Transfer successful, create deposit entry
            let current_time = ic_cdk::api::time();
            let unlock_time = current_time + (timelock * 1_000_000_000); // Convert seconds to nanoseconds
            
            let deposit = types::Deposit {
                unlocktime: unlock_time,
                principal: user,
                amount: transfer_amount.0.try_into().map_err(|_| types::DepositError::InternalError)?,
            };

            // Get current counter and increment it
            let deposit_id = DEPOSIT_COUNTER.with(|counter| {
                let current = counter.borrow().get().clone();
                let new_counter = current + 1;
                counter.borrow_mut().set(new_counter).ok();
                current
            });

            // Store deposit with counter as key
            let deposit_json = serde_json::to_string(&deposit).expect("Failed to serialize");
            DEPOSITS.with(|deposits| {
                deposits.borrow_mut().insert(deposit_id, deposit_json);
            });

            Ok(())
        },
        Ok((Err(_transfer_error),)) => Err(types::DepositError::TransferFailed),
        Err(_) => Err(types::DepositError::LedgerCallFailed),
    }
}

ic_cdk::export_candid!();
