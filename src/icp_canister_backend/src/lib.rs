mod types;

use candid::{Nat, Principal};
use sha2::{Sha256, Digest};
use std::cell::RefCell;
use ic_cdk::api::call::call;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, StableCell};


use types::{Account, Deposit, DepositError};


type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static DEPOSITS: RefCell<StableBTreeMap<String, String, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
        )
    );

    static LEDGER_CANISTER_ID: RefCell<StableCell<String, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))),
            String::new()
        ).expect("Failed to initialize StableCell")
    );
}


fn generate_subaccount(user: Principal, timelock: u64) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(user.as_slice());
    hasher.update(timelock.to_be_bytes());
    hasher.finalize().into()
}


#[ic_cdk::query]
pub fn get_deposit_subaccount(user: Principal, timelock: u64) -> [u8; 32] {
    generate_subaccount(user, timelock)
}

#[ic_cdk::update]
pub fn set_ledger_canister_id(ledger_id: Principal) {
    LEDGER_CANISTER_ID.with(|cell| {
        cell.borrow_mut().set(ledger_id.to_text()).ok();
    });
}

#[ic_cdk::update]
pub async fn deposit(user: Principal, timelock: u64) -> Result<(), DepositError> {
    if timelock == 0 {
        return Err(DepositError::InvalidTimelock);
    }
    let ledger_principal = LEDGER_CANISTER_ID.with(|cell| {
        let binding = cell.borrow();
        let stored = binding.get().clone(); 
        if stored.is_empty() {
            None
        } else {
            Principal::from_text(&stored).ok()
        }
    }).ok_or(DepositError::LedgerNotSet)?;

    let deposit_key = format!("{}:{}", user.to_text(), timelock);

    let exists = DEPOSITS.with(|deposits| {
        deposits.borrow().contains_key(&deposit_key)
    });

    if exists {
        return Err(DepositError::DepositAlreadyExists);
    }
    
    // Generate subaccount and check balance
    let subaccount = generate_subaccount(user, timelock);
    let account = Account {
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
        Err(_) => return Err(DepositError::LedgerCallFailed),
    };

    let balance_u64: u64 = balance.0.try_into().map_err(|_| DepositError::InternalError)?;

    if balance_u64 == 0 {
        return Err(DepositError::InsufficientBalance);
    }
    let deposit = Deposit {
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
