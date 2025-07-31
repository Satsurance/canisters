use candid::Principal;
use candid::{CandidType, Deserialize, Nat};
use ic_stable_structures::storable::Bound;
use ic_stable_structures::Storable;
use serde::Serialize;
use std::borrow::Cow;
use std::vec::Vec;

#[derive(CandidType, Deserialize, Debug)]
pub enum TransferError {
    BadFee { expected_fee: Nat },
    BadBurn { min_burn_amount: Nat },
    InsufficientFunds { balance: Nat },
    TooOld,
    CreatedInFuture { ledger_time: u64 },
    TemporarilyUnavailable,
    Duplicate { duplicate_of: Nat },
    GenericError { error_code: Nat, message: String },
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Account {
    pub owner: Principal,
    pub subaccount: Option<Vec<u8>>,
}

#[derive(CandidType, Deserialize, Clone, Debug, Serialize)]
pub struct Deposit {
    pub unlocktime: u64,
    pub shares: Nat,
}

impl Storable for Deposit {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 80,
        is_fixed_size: false,
    };
}

#[derive(CandidType, Deserialize, Debug)]
pub struct TransferArg {
    pub from_subaccount: Option<Vec<u8>>,
    pub to: Account,
    pub amount: Nat,
    pub fee: Option<Nat>,
    pub memo: Option<Vec<u8>>,
    pub created_at_time: Option<u64>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct UserDepositInfo {
    pub deposit_id: u64,
    pub shares: Nat,
    pub amount: Nat,
    pub unlock_time: u64,
}

#[derive(CandidType, Deserialize, Debug)]
pub enum PoolError {
    NoDeposit,
    InsufficientBalance,
    InvalidTimelock,
    TransferFailed,
    LedgerCallFailed,
    InternalError,
    LedgerNotSet,
    DepositAlreadyExists,
    NotOwner,
    TimelockNotExpired,
}

#[derive(Clone, Debug)]
pub struct UserDeposits(pub Vec<u64>);

impl Storable for UserDeposits {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(candid::encode_one(&self.0).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        UserDeposits(candid::decode_one(&bytes).unwrap())
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[derive(CandidType, Deserialize, Clone, Debug, Serialize)]
pub struct PoolState {
    pub total_assets: Nat,
    pub total_shares: Nat,
}

impl Storable for PoolState {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 100,
        is_fixed_size: false,
    };
}
