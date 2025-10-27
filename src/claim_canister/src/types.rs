use candid::{CandidType, Deserialize, Nat, Principal};
use ic_stable_structures::memory_manager::VirtualMemory;
use ic_stable_structures::{DefaultMemoryImpl, storable::{Bound, Storable}};
use serde::Serialize;
use std::borrow::Cow;

pub type Memory = VirtualMemory<DefaultMemoryImpl>;

#[derive(CandidType, Deserialize, Clone, Debug, Serialize)]
pub struct StorableNat(pub Nat);

impl Storable for StorableNat {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(candid::encode_one(&self.0).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        StorableNat(candid::decode_one(&bytes).unwrap())
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[derive(CandidType, Deserialize, Debug)]
pub struct Account {
    pub owner: Principal,
    pub subaccount: Option<Vec<u8>>,
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

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Serialize)]
pub enum ClaimStatus {
    Pending,
    Approved,
    Executing,
    Executed,
    Rejected,
}

#[derive(CandidType, Deserialize, Clone, Debug, Serialize)]
pub struct Claim {
    pub id: u64,
    pub proposer: Principal,
    pub receiver: Principal,
    pub amount: Nat,
    pub pool_canister_id: Principal,
    pub description: String,
    pub status: ClaimStatus,
    pub created_at: u64,
    pub approved_at: Option<u64>,
    pub approved_by: Option<Principal>,
    pub deposit_amount: Nat,
    pub spam: bool,
}

impl Storable for Claim {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[derive(CandidType, Deserialize, Debug, PartialEq)]
pub enum ClaimError {
    NotFound,
    NotApprover,
    AlreadyApproved,
    AlreadyExecuting,
    AlreadyExecuted,
    NotApproved,
    TimelockNotExpired,
    ExecutionTimeoutNotExpired,
    ApprovalPeriodExpired,
    PoolCallFailed(String),
    InsufficientPermissions,
    NotProposer,
    AlreadyMarkedAsSpam,
    NoDepositToWithdraw,
    DepositTransferFailed,
    InsufficientDeposit,
    LedgerNotSet,
}

#[derive(CandidType, Deserialize, Clone, Debug, Serialize)]
pub struct ClaimInfo {
    pub id: u64,
    pub proposer: Principal,
    pub receiver: Principal,
    pub amount: Nat,
    pub pool_canister_id: Principal,
    pub description: String,
    pub status: ClaimStatus,
    pub created_at: u64,
    pub approved_at: Option<u64>,
    pub approved_by: Option<Principal>,
    pub deposit_amount: Nat,
    pub spam: bool,
}

#[derive(CandidType, Deserialize, Debug, PartialEq)]
pub enum PoolError {
    NoDeposit,
    InsufficientBalance,
    TransferFailed,
    LedgerCallFailed,
    LedgerNotSet,
    NotOwner,
    TimelockNotExpired,
    EpisodeNotActive,
    EpisodeNotStakable,
    NotSlashingExecutor,
}
