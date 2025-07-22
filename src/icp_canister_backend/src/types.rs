use candid::{CandidType, Deserialize};
use candid::Principal;
use serde::Serialize;

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
