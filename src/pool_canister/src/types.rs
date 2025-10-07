use candid::Principal;
use candid::{CandidType, Deserialize, Nat};
use ic_stable_structures::storable::Bound;
use ic_stable_structures::Storable;
use serde::Serialize;
use std::borrow::Cow;

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

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CoverageInfo {
    pub coverage_id: u64,
    pub product_id: u64,
    pub covered_account: Principal,
    pub coverage_amount: Nat,
    pub premium_amount: Nat,
    pub start_time: u64,
    pub end_time: u64,
}
