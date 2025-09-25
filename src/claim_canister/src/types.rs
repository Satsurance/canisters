use candid::{CandidType, Deserialize, Nat, Principal};
use ic_stable_structures::storable::{Bound, Storable};
use serde::Serialize;
use std::borrow::Cow;

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
    pub receiver: Principal,
    pub amount: Nat,
    pub pool_canister_id: Principal,
    pub description: String,
    pub status: ClaimStatus,
    pub created_at: u64,
    pub approved_at: Option<u64>,
    pub approved_by: Option<Principal>,
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
    PoolCallFailed,
    InsufficientPermissions,
    InvalidStatus,
}

#[derive(CandidType, Deserialize, Clone, Debug, Serialize)]
pub struct ClaimInfo {
    pub id: u64,
    pub receiver: Principal,
    pub amount: Nat,
    pub pool_canister_id: Principal,
    pub description: String,
    pub status: ClaimStatus,
    pub created_at: u64,
    pub approved_at: Option<u64>,
    pub approved_by: Option<Principal>,
    pub can_execute: bool,
    pub time_until_execution: Option<u64>,
}
