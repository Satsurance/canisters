use candid::Principal;
use candid::{CandidType, Deserialize, Nat};
use ic_stable_structures::storable::Bound;
use ic_stable_structures::Storable;
use serde::Serialize;
use std::borrow::Cow;

// Re-export shared types from commons
pub use commons::types::{Account, StorableNat, TransferArg, TransferError};

#[derive(CandidType, Deserialize, Clone, Debug, Serialize)]
pub struct Deposit {
    pub episode: u64,
    pub shares: Nat,
    pub reward_per_share: Nat,
    pub rewards_collected: Nat,
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

#[derive(CandidType, Deserialize, Clone, Debug, Serialize)]
pub struct Episode {
    pub episode_shares: Nat,
    pub assets_staked: Nat,
    pub reward_decrease: Nat,
    pub coverage_decrease: Nat,
    pub acc_reward_per_share_on_expire: Nat,
}

impl Storable for Episode {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 120,
        is_fixed_size: false,
    };
}

#[derive(CandidType, Deserialize, Debug)]
pub struct UserDepositInfo {
    pub deposit_id: u64,
    pub episode: u64,
    pub shares: Nat,
    pub amount: Nat,
}

#[derive(CandidType, Deserialize, Debug)]
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
    NotPoolManager,
    ProductNotActive,
    CoverageDurationTooLong,
    CoverageDurationTooShort,
    NotEnoughAssetsToCover,
    ProductNotFound,
    InvalidProductParameters,
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

#[derive(CandidType, Deserialize, Clone, Debug, Serialize)]
pub struct Product {
    pub name: String,
    pub product_id: u64,
    pub annual_percent: u64,
    pub max_coverage_duration: u64,
    pub max_pool_allocation_percent: u64,
    pub allocation: Nat,
    pub last_allocation_update: u64,
    pub active: bool,
}

impl Storable for Product {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[derive(CandidType, Deserialize, Clone, Debug, Serialize)]
pub struct Coverage {
    pub coverage_id: u64,
    pub buyer: Principal,
    pub covered_account: Principal,
    pub product_id: u64,
    pub coverage_amount: Nat,
    pub premium_amount: Nat,
    pub start_time: u64,
    pub end_time: u64,
}

impl Storable for Coverage {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(candid::encode_one(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        candid::decode_one(&bytes).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[derive(Clone, Debug)]
pub struct UserCoverages(pub Vec<u64>);

impl Storable for UserCoverages {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(candid::encode_one(&self.0).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        UserCoverages(candid::decode_one(&bytes).unwrap())
    }

    const BOUND: Bound = Bound::Unbounded;
}
