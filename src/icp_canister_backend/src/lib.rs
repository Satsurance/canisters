use candid::{Nat, Principal};
use ic_cdk::api::call::call;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, StableCell};
use lazy_static::lazy_static;
use sha2::{Digest, Sha256};
use std::cell::RefCell;

lazy_static! {
    pub static ref TRANSFER_FEE: Nat = Nat::from(10_000u64);
    pub static ref MINIMUM_DEPOSIT_AMOUNT: Nat = Nat::from(100_000u64);
}

pub mod types;

pub use types::{Account, Deposit, PoolError, TransferArg, TransferError};

type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static TOKEN_ID: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
            Principal::anonymous()
        ).expect("Failed to initialize StableCell")
    );

    static DEPOSIT_COUNTER: RefCell<StableCell<u64, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))),
            0u64
        ).expect("Failed to initialize StableCell")
    );

    static DEPOSITS: RefCell<StableBTreeMap<u64, Deposit, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2))),
        )
    );

    static USER_DEPOSITS: RefCell<StableBTreeMap<Principal, types::UserDeposits, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3))),
        )
    );
}

#[ic_cdk::query]
pub fn get_deposit_subaccount(user: Principal, timelock: u64) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(user.as_slice());
    hasher.update(timelock.to_be_bytes());
    hasher.finalize().into()
}

#[ic_cdk::query]
pub fn get_user_deposits(user: Principal) -> Vec<types::UserDepositInfo> {
    let now = ic_cdk::api::time();

    USER_DEPOSITS.with(|user_deposits| {
        user_deposits
            .borrow()
            .get(&user)
            .map(|deposits| {
                deposits
                    .0
                    .iter()
                    .filter_map(|&deposit_id| {
                        DEPOSITS.with(|deposits| {
                            deposits.borrow().get(&deposit_id).map(|deposit| {
                                types::UserDepositInfo {
                                    deposit_id,
                                    amount: deposit.amount.clone(),
                                    unlock_time: deposit.unlocktime,
                                    is_expired: now >= deposit.unlocktime,
                                }
                            })
                        })
                    })
                    .collect()
            })
            .unwrap_or(vec![])
    })
}

#[ic_cdk::init]
pub fn init(token_id: Principal) {
    TOKEN_ID.with(|cell| {
        cell.borrow_mut().set(token_id).ok();
    });
}

#[ic_cdk::update]
pub async fn deposit(user: Principal, timelock: u64) -> Result<(), types::PoolError> {
    if timelock == 0 {
        return Err(types::PoolError::InvalidTimelock);
    }

    let ledger_principal = TOKEN_ID
        .with(|cell| {
            let stored = cell.borrow().get().clone();
            Some(stored)
        })
        .ok_or(types::PoolError::LedgerNotSet)?;

    let subaccount = get_deposit_subaccount(user, timelock);
    let from_account = types::Account {
        owner: ic_cdk::api::id(),
        subaccount: Some(subaccount.to_vec()),
    };

    let balance_result: Result<(Nat,), _> =
        call(ledger_principal, "icrc1_balance_of", (from_account,)).await;

    let balance = match balance_result {
        Ok((balance,)) => balance,
        Err(_) => return Err(types::PoolError::LedgerCallFailed),
    };

    let transfer_amount = if balance.0 > MINIMUM_DEPOSIT_AMOUNT.0 {
        Nat::from(&balance.0 - &TRANSFER_FEE.0)
    } else {
        return Err(types::PoolError::InsufficientBalance);
    };

    let transfer_args = (types::TransferArg {
        from_subaccount: Some(subaccount.to_vec()),
        to: types::Account {
            owner: ic_cdk::api::id(),
            subaccount: None,
        },
        amount: transfer_amount.clone(),
        fee: Some(TRANSFER_FEE.clone()),
        memo: None,
        created_at_time: None,
    },);

    let transfer_result: Result<(Result<Nat, types::TransferError>,), _> =
        call(ledger_principal, "icrc1_transfer", transfer_args).await;

    if transfer_result.is_err() || transfer_result.as_ref().unwrap().0.is_err() {
        return Err(types::PoolError::TransferFailed);
    }

    let current_time = ic_cdk::api::time();
    let unlock_time = current_time + (timelock * 1_000_000_000);
    let deposit_id = DEPOSIT_COUNTER.with(|counter| {
        let current = counter.borrow().get().clone();
        let new_counter = current + 1;
        counter.borrow_mut().set(new_counter).ok();
        current
    });
    let deposit = types::Deposit {
        unlocktime: unlock_time,
        amount: transfer_amount.clone(),
    };

    DEPOSITS.with(|deposits| {
        deposits.borrow_mut().insert(deposit_id, deposit);
    });

    USER_DEPOSITS.with(|user_deposits| {
        let mut user_deposits = user_deposits.borrow_mut();
        let user_deposits_list = user_deposits
            .get(&user)
            .unwrap_or(types::UserDeposits(vec![]));
        let mut new_deposits = user_deposits_list.0.clone();
        new_deposits.push(deposit_id);
        user_deposits.insert(user, types::UserDeposits(new_deposits));
    });

    Ok(())
}

#[ic_cdk::update]
pub async fn withdraw(deposit_id: u64) -> Result<(), types::PoolError> {
    let caller = ic_cdk::api::caller();
    let now = ic_cdk::api::time();

    let deposit = DEPOSITS.with(|deposits| deposits.borrow().get(&deposit_id).clone());
    let deposit = match deposit {
        Some(d) => d,
        None => return Err(types::PoolError::NoDeposit),
    };

    let user_deposits =
        USER_DEPOSITS.with(|user_deposits| user_deposits.borrow().get(&caller).clone());

    let user_deposits = match user_deposits {
        Some(deposits) => deposits,
        None => return Err(types::PoolError::NotOwner),
    };

    if !user_deposits.0.contains(&deposit_id) {
        return Err(types::PoolError::NotOwner);
    }

    if now < deposit.unlocktime {
        return Err(types::PoolError::TimelockNotExpired);
    }

    DEPOSITS.with(|deposits| deposits.borrow_mut().remove(&deposit_id));

    USER_DEPOSITS.with(|user_deposits| {
        let mut user_deposits = user_deposits.borrow_mut();
        if let Some(user_deposits_list) = user_deposits.get(&caller) {
            let mut new_deposits = user_deposits_list.0.clone();
            new_deposits.retain(|&id| id != deposit_id);
            user_deposits.insert(caller, types::UserDeposits(new_deposits));
        }
    });

    let ledger_principal = TOKEN_ID.with(|cell| cell.borrow().get().clone());
    let transfer_amount = Nat::from(&deposit.amount.0 - &TRANSFER_FEE.0);
    let transfer_args = (types::TransferArg {
        from_subaccount: None,
        to: types::Account {
            owner: caller,
            subaccount: None,
        },
        amount: transfer_amount,
        fee: Some(TRANSFER_FEE.clone()),
        memo: None,
        created_at_time: None,
    },);
    let transfer_result: Result<(Result<Nat, types::TransferError>,), _> =
        call(ledger_principal, "icrc1_transfer", transfer_args).await;

    if transfer_result.is_err() || transfer_result.as_ref().unwrap().0.is_err() {
        DEPOSITS.with(|deposits| deposits.borrow_mut().insert(deposit_id, deposit));

        USER_DEPOSITS.with(|user_deposits| {
            let mut user_deposits = user_deposits.borrow_mut();
            let user_deposits_list = user_deposits
                .get(&caller)
                .unwrap_or(types::UserDeposits(vec![]));
            let mut new_deposits = user_deposits_list.0.clone();
            new_deposits.push(deposit_id);
            user_deposits.insert(caller, types::UserDeposits(new_deposits));
        });

        return Err(types::PoolError::TransferFailed);
    }

    Ok(())
}

ic_cdk::export_candid!();
