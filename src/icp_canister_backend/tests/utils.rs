#![allow(dead_code)]
#[path = "setup.rs"]
pub mod setup;
use candid::{Nat, Principal};
use icp_canister_backend::Account;
use setup::client::{PoolCanister, TransferResult};

lazy_static::lazy_static! {
    pub static ref ALLOWED_ERROR: Nat = Nat::from(10u64);
    pub static ref TRANSFER_FEE: Nat = Nat::from(10_000u64);
}

pub fn create_deposit(client: &mut PoolCanister, user: Principal, amount: Nat, episode: u64) {
    let subaccount = client.get_deposit_subaccount(user, episode);

    let transfer_args = icp_canister_backend::TransferArg {
        from_subaccount: None,
        to: Account {
            owner: client.canister_id,
            subaccount: Some(subaccount.to_vec()),
        },
        amount: amount.clone(),
        fee: Some(TRANSFER_FEE.clone()),
        memo: None,
        created_at_time: None,
    };

    let transfer_result = client.connect(user).icrc1_transfer(transfer_args);
    assert!(
        matches!(transfer_result, TransferResult::Ok(_)),
        "Transfer should succeed"
    );

    let result = client.deposit(user, episode);
    assert!(result.is_ok(), "Deposit should succeed: {:?}", result);
}

pub fn get_current_episode(client: &PoolCanister) -> u64 {
    client.get_current_episode_id()
}

pub fn get_stakable_episode(client: &PoolCanister, relative_episode: u8) -> u64 {
    if relative_episode > 7 {
        panic!("Relative episode must be 0-7");
    }

    let current_episode = client.get_current_episode_id();
    let mut first_stakable = current_episode;
    while first_stakable % 3 != 2 {
        first_stakable += 1;
    }
    first_stakable + (relative_episode as u64 * 3)
}

pub fn get_episode_time_to_end(client: &PoolCanister, target_episode: u64) -> u64 {
    let target_episode_end_time = (target_episode + 1) * icp_canister_backend::EPISODE_DURATION;
    let current_time = client.pic.get_time().as_nanos_since_unix_epoch() / 1_000_000_000;
    target_episode_end_time - current_time
}

pub fn advance_time(client: &PoolCanister, duration_seconds: u64) {
    client
        .pic
        .advance_time(std::time::Duration::from_secs(duration_seconds));
    client.pic.tick();
}

pub fn get_current_time(client: &PoolCanister) -> u64 {
    client.pic.get_time().as_nanos_since_unix_epoch() / 1_000_000_000
}

pub fn reward_pool(client: &mut PoolCanister, user: Principal, reward_amount: Nat) -> Result<(), String> {
    let reward_subaccount = client.get_reward_subaccount();

    let transfer_args = icp_canister_backend::TransferArg {
        from_subaccount: None,
        to: Account {
            owner: client.canister_id,
            subaccount: Some(reward_subaccount.to_vec()),
        },
        amount: reward_amount + TRANSFER_FEE.clone(),
        fee: Some(TRANSFER_FEE.clone()),
        memo: None,
        created_at_time: None,
    };

    let transfer_result = client.connect(user).icrc1_transfer(transfer_args);
    if let TransferResult::Err(e) = transfer_result {
        return Err(format!("Transfer failed: {:?}", e));
    }

    let result = client.reward_pool();
    result.map_err(|e| format!("Reward pool failed: {:?}", e))
}

#[macro_export]
macro_rules! assert_with_error {
    ($actual:expr, $expected:expr, $allowed_error:expr, $message:expr) => {{
        use candid::Nat;
        let actual_val: Nat = (*$actual).clone();
        let expected_val: Nat = (*$expected).clone();
        let allowed_error_val: Nat = (*$allowed_error).clone();

        let diff: Nat = if actual_val > expected_val {
            actual_val.clone() - expected_val.clone()
        } else {
            expected_val.clone() - actual_val.clone()
        };

        assert!(
            diff <= allowed_error_val,
            "{}: expected {}, got {}, error {}, allowed error {}",
            $message,
            expected_val,
            actual_val,
            diff,
            allowed_error_val
        );
    }};
}
