use candid::{Nat, Principal};
use sha2::{Sha256, Digest};
use std::cell::RefCell;
use ic_cdk::api::call::call;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, StableCell};
use serde_json;

pub mod types;
// Re-export for external use
pub use types::{Account, Deposit, DepositError};

type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static DEPOSITS: RefCell<StableBTreeMap<String, String, Memory>> = RefCell::new(
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

    let deposit_key = format!("{}:{}", user.to_text(), timelock);

    let exists = DEPOSITS.with(|deposits| {
        deposits.borrow().contains_key(&deposit_key)
    });

    if exists {
        return Err(types::DepositError::DepositAlreadyExists);
    }
    
    // Generate subaccount and check balance
    let subaccount = get_deposit_subaccount(user, timelock);
    let account = types::Account {
        owner: ic_cdk::api::id(),
        subaccount: Some(subaccount.to_vec()),
    };

    let balance_result: Result<(Nat,), _> = call(
        ledger_principal,
        "icrc1_balance_of",
        (account,)
    ).await;

    let balance = match balance_result {
        Ok((balance,)) => balance,
        Err(_) => return Err(types::DepositError::LedgerCallFailed),
    };

    let balance_u64: u64 = balance.0.try_into().map_err(|_| types::DepositError::InternalError)?;

    if balance_u64 == 0 {
        return Err(types::DepositError::InsufficientBalance);
    }
    
    let deposit = types::Deposit {
        deposit_time: ic_cdk::api::time(),
        user,
        subaccount: subaccount.to_vec(),
        amount: balance_u64,
        timelock,
    };
    
    let deposit_json = serde_json::to_string(&deposit).expect("Failed to serialize");
    DEPOSITS.with(|deposits| {
        deposits.borrow_mut().insert(deposit_key, deposit_json);
    });

    Ok(())
}

ic_cdk::export_candid!();
