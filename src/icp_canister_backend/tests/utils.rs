#![allow(dead_code)]
#[path = "types.rs"]
mod types;
use candid::{decode_one, encode_args, Nat, Principal};
use icp_canister_backend::{Account, PoolError};
use pocket_ic::PocketIc;
use types::TransferResult;

lazy_static::lazy_static! {
    pub static ref ALLOWED_ERROR: Nat = Nat::from(10u64);
    pub static ref TRANSFER_FEE: Nat = Nat::from(10_000u64);
}

pub fn reward_pool(
    pic: &PocketIc,
    canister_id: Principal,
    ledger_id: Principal,
    user: Principal,
    reward_amount: Nat,
) -> Result<(), String> {
    let reward_subaccount_result = pic
        .query_call(
            canister_id,
            user,
            "get_reward_subaccount",
            encode_args(()).unwrap(),
        )
        .map_err(|e| format!("Failed to get reward subaccount: {:?}", e))?;
    let reward_subaccount: [u8; 32] = decode_one(&reward_subaccount_result)
        .map_err(|e| format!("Failed to decode reward subaccount: {:?}", e))?;

    let transfer_args = icp_canister_backend::TransferArg {
        from_subaccount: None,
        to: Account {
            owner: canister_id,
            subaccount: Some(reward_subaccount.to_vec()),
        },
        amount: reward_amount + TRANSFER_FEE.clone(),
        fee: Some(TRANSFER_FEE.clone()),
        memo: None,
        created_at_time: None,
    };
    let transfer_result = pic
        .update_call(
            ledger_id,
            user,
            "icrc1_transfer",
            encode_args((transfer_args,)).unwrap(),
        )
        .map_err(|e| format!("Failed to transfer reward tokens: {:?}", e))?;

    let transfer_result: TransferResult = decode_one(&transfer_result)
        .map_err(|e| format!("Failed to decode transfer result: {:?}", e))?;

    if let TransferResult::Err(e) = transfer_result {
        return Err(format!("Transfer failed: {:?}", e));
    }

    let reward_result = pic
        .update_call(canister_id, user, "reward_pool", encode_args(()).unwrap())
        .map_err(|e| format!("Failed to call reward_pool: {:?}", e))?;

    let result: Result<(), PoolError> = decode_one(&reward_result)
        .map_err(|e| format!("Failed to decode reward_pool result: {:?}", e))?;

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
