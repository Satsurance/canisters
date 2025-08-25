use candid::{Nat, Principal};
use lazy_static::lazy_static;

pub const EPISODE_DURATION: u64 = 91 * 24 * 60 * 60 / 3;
const MAX_ACTIVE_EPISODES: u64 = 24;

lazy_static! {
    pub static ref TRANSFER_FEE: Nat = Nat::from(10_000u64);
    pub static ref MINIMUM_DEPOSIT_AMOUNT: Nat = Nat::from(100_000u64);
    pub static ref PRECISION_SCALE: Nat = Nat::from(1_000_000_000_000_000_000u64);
}

pub mod deposit;
pub mod episodes;
pub mod governance;
pub mod ledger;
pub mod rewards;
pub mod storage;
pub mod types;

pub use types::{
    Account, Deposit, Episode, PoolError, PoolState, StorableNat, TransferArg, TransferError,
    UserDepositInfo, UserDeposits,
};

pub use ledger::{calculate_net_amount, get_subaccount_balance, transfer_icrc1};
use storage::*;

use episodes::setup_episode_timer;

#[ic_cdk::init]
pub fn init(token_id: Principal, executor: Principal) {
    TOKEN_ID.with(|cell| {
        cell.borrow_mut().set(token_id).ok();
    });

    let current_time = ic_cdk::api::time() / 1_000_000_000;
    LAST_TIME_UPDATED.with(|cell| {
        cell.borrow_mut().set(current_time).ok();
    });

    EXECUTOR_PRINCIPAL.with(|cell| {
        cell.borrow_mut().set(executor).ok();
    });

    setup_episode_timer();
}

ic_cdk::export_candid!();
