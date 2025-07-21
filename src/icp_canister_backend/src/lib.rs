use candid::{Nat, Principal};
use serde::Serialize;
use sha2::{Sha256, Digest};
use std::cell::RefCell;
use ic_cdk::api::call::call;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, StableCell};
use candid::{CandidType, Deserialize};

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Account {
    pub owner: Principal,
    pub subaccount: Option<Vec<u8>>,
}

#[derive(CandidType, Deserialize, Clone, Debug, Serialize)]
pub struct Deposit {
    pub deposit_time: u64,
    pub user: Principal,
    pub subaccount: Vec<u8>,
    pub amount: u64,
    pub timelock: u64,
}

#[derive(CandidType, Deserialize, Debug)]
pub enum DepositError {
    NoDeposit,
    InsufficientBalance,
    InvalidTimelock,
    TransferFailed,
    LedgerCallFailed,
    InternalError,
    LedgerNotSet,
    DepositAlreadyExists,
}


type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static DEPOSITS: RefCell<StableBTreeMap<String, String, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
        )
    );

    static TOKEN_ID: RefCell<StableCell<Vec<u8>, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))),
            Vec::new()
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
            cell.borrow_mut().set(id.as_slice().to_vec()).ok();
        });
    }
}

#[ic_cdk::update]
pub fn set_token_id(token_id: Principal) {
    TOKEN_ID.with(|cell| {
        cell.borrow_mut().set(token_id.as_slice().to_vec()).ok();
    });
}

#[ic_cdk::update]
pub async fn deposit(user: Principal, timelock: u64) -> Result<(), DepositError> {
    if timelock == 0 {
        return Err(DepositError::InvalidTimelock);
    }
    let ledger_principal = TOKEN_ID.with(|cell| {
        let binding = cell.borrow();
        let stored = binding.get().clone(); 
        if stored.is_empty() {
            None
        } else {
            Some(Principal::from_slice(&stored))
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
    let subaccount = get_deposit_subaccount(user, timelock);
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
