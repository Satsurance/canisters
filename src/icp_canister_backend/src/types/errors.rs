use candid::{CandidType, Deserialize};

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
