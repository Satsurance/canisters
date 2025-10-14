use crate::clients::{LedgerCanisterClient, PoolCanisterClient};
use candid::{decode_one, encode_args, Nat, Principal};
use pocket_ic::PocketIc;
use pool_canister::{Account, TransferArg};

// Constants
lazy_static::lazy_static! {
    pub static ref ALLOWED_ERROR: Nat = Nat::from(10u64);
    pub static ref TRANSFER_FEE: Nat = Nat::from(10u64);
}

pub fn get_stakable_episode(pic: &PocketIc, pool_canister: Principal, caller: Principal) -> u64 {
    let current_episode_bytes = pic
        .query_call(
            pool_canister,
            caller,
            "get_current_episode_id",
            encode_args(()).unwrap(),
        )
        .unwrap();
    let mut current_episode: u64 = decode_one(&current_episode_bytes).unwrap();

    while current_episode % 3 != 2 {
        current_episode += 1;
    }

    current_episode
}

pub fn get_stakable_episode_with_client(client: &PoolCanisterClient, relative_episode: u8) -> u64 {
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

pub fn create_deposit(
    pool_client: &mut PoolCanisterClient,
    ledger_client: &mut LedgerCanisterClient,
    user: Principal,
    amount: Nat,
    episode: u64,
) -> Result<(), String> {
    let subaccount = pool_client
        .connect(user)
        .get_deposit_subaccount(user, episode);

    let transfer_args = TransferArg {
        from_subaccount: None,
        to: Account {
            owner: pool_client.client.canister_id,
            subaccount: Some(subaccount.to_vec()),
        },
        amount: amount.clone(),
        fee: Some(TRANSFER_FEE.clone()),
        memo: None,
        created_at_time: None,
    };

    let transfer_result = ledger_client.connect(user).icrc1_transfer(transfer_args);
    match transfer_result {
        crate::clients::ledger::TransferResult::Ok(_) => {}
        crate::clients::ledger::TransferResult::Err(e) => {
            return Err(format!("Transfer failed: {:?}", e));
        }
    }

    let result = pool_client.deposit(user, episode);
    match result {
        Ok(()) => Ok(()),
        Err(e) => Err(format!("Deposit failed: {:?}", e)),
    }
}

pub fn reward_pool(
    pool_client: &mut PoolCanisterClient,
    ledger_client: &mut LedgerCanisterClient,
    user: Principal,
    reward_amount: Nat,
) -> Result<(), String> {
    let reward_subaccount = pool_client.get_reward_subaccount();

    let transfer_args = TransferArg {
        from_subaccount: None,
        to: Account {
            owner: pool_client.client.canister_id,
            subaccount: Some(reward_subaccount.to_vec()),
        },
        amount: reward_amount + TRANSFER_FEE.clone(),
        fee: Some(TRANSFER_FEE.clone()),
        memo: None,
        created_at_time: None,
    };

    let transfer_result = ledger_client.connect(user).icrc1_transfer(transfer_args);
    match transfer_result {
        crate::clients::ledger::TransferResult::Ok(_) => {}
        crate::clients::ledger::TransferResult::Err(e) => {
            return Err(format!("Transfer failed: {:?}", e));
        }
    }

    let result = pool_client.reward_pool();
    match result {
        Ok(()) => Ok(()),
        Err(e) => Err(format!("Reward pool failed: {:?}", e)),
    }
}

pub fn purchase_coverage(
    pool_client: &mut PoolCanisterClient,
    ledger_client: &mut LedgerCanisterClient,
    buyer: Principal,
    product_id: u64,
    covered_account: Principal,
    coverage_duration: u64,
    coverage_amount: Nat,
    premium_amount: Nat,
) -> Result<(), String> {
    let subaccount = pool_client
        .connect(buyer)
        .get_purchase_subaccount(buyer, product_id);

    let transfer_args = TransferArg {
        from_subaccount: None,
        to: Account {
            owner: pool_client.client.canister_id,
            subaccount: Some(subaccount.to_vec()),
        },
        amount: premium_amount.clone(),
        fee: Some(TRANSFER_FEE.clone()),
        memo: None,
        created_at_time: None,
    };

    let transfer_result = ledger_client.connect(buyer).icrc1_transfer(transfer_args);
    match transfer_result {
        crate::clients::ledger::TransferResult::Ok(_) => {}
        crate::clients::ledger::TransferResult::Err(e) => {
            return Err(format!("Transfer to subaccount failed: {:?}", e));
        }
    }

    let result = pool_client.purchase_coverage(
        product_id,
        covered_account,
        coverage_duration,
        coverage_amount,
    );
    match result {
        Ok(()) => Ok(()),
        Err(e) => Err(format!("Coverage purchase failed: {:?}", e)),
    }
}

pub fn advance_time(pic: &PocketIc, duration_seconds: u64) {
    pic.advance_time(std::time::Duration::from_secs(duration_seconds));
    pic.tick();
}

pub fn get_current_time(pic: &PocketIc) -> u64 {
    pic.get_time().as_nanos_since_unix_epoch() / 1_000_000_000
}

pub fn get_episode_time_to_end(client: &PoolCanisterClient, target_episode: u64) -> u64 {
    let target_episode_end_time = (target_episode + 1) * pool_canister::EPISODE_DURATION;
    let current_time = client.client.pic.get_time().as_nanos_since_unix_epoch() / 1_000_000_000;
    target_episode_end_time - current_time
}

pub fn calculate_premium(
    coverage_duration: u64,
    annual_percent: u64,
    coverage_amount: Nat,
) -> Nat {
    const BASIS_POINTS: u64 = 10_000;
    const SECONDS_PER_YEAR: u64 = 365 * 24 * 60 * 60;
    
    (Nat::from(coverage_duration) * Nat::from(annual_percent) * coverage_amount)
        / (Nat::from(SECONDS_PER_YEAR) * Nat::from(BASIS_POINTS))
}

/// Macro for assertions with allowed error
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
