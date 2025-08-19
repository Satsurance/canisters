use candid::{decode_one, encode_args, Nat, Principal};
use icp_canister_backend::{Account, PoolError};
use pocket_ic::PocketIc;

#[derive(candid::CandidType, candid::Deserialize, Debug)]
pub enum TransferResult {
    Ok(Nat),
    Err(icp_canister_backend::TransferError),
}

lazy_static::lazy_static! {
    pub static ref TRANSFER_FEE: Nat = Nat::from(10_000u64);
}

pub fn create_deposit(
    pic: &PocketIc,
    canister_id: Principal,
    ledger_id: Principal,
    user: Principal,
    amount: Nat,
    episode: u64,
) {
    let subaccount_result = pic
        .query_call(
            canister_id,
            user,
            "get_deposit_subaccount",
            encode_args((user, episode)).unwrap(),
        )
        .expect("Failed to get deposit subaccount");
    let subaccount: [u8; 32] = candid::decode_one(&subaccount_result).unwrap();

    let transfer_args = icp_canister_backend::TransferArg {
        from_subaccount: None,
        to: Account {
            owner: canister_id,
            subaccount: Some(subaccount.to_vec()),
        },
        amount: amount.clone(),
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
        .expect("Failed to transfer tokens");

    let transfer_result: TransferResult = candid::decode_one(&transfer_result).unwrap();
    assert!(
        matches!(transfer_result, TransferResult::Ok(_)),
        "Transfer should succeed"
    );

    let deposit_result = pic
        .update_call(
            canister_id,
            user,
            "deposit",
            encode_args((user, episode)).unwrap(),
        )
        .expect("Failed to call deposit");

    let result: Result<(), PoolError> = candid::decode_one(&deposit_result).unwrap();
    assert!(
        matches!(result, Ok(_)),
        "Deposit should succeed: {:?}",
        result
    );
}

pub fn get_current_episode(pic: &PocketIc, canister_id: Principal) -> u64 {
    let current_episode_result = pic
        .query_call(
            canister_id,
            Principal::anonymous(),
            "get_current_episode_id",
            encode_args(()).unwrap(),
        )
        .expect("Failed to get current episode");
    candid::decode_one(&current_episode_result).unwrap()
}

pub fn get_stakable_episode(pic: &PocketIc, canister_id: Principal, relative_episode: u8) -> u64 {
    if relative_episode > 7 {
        panic!("Relative episode must be 0-7");
    }

    let current_episode = get_current_episode(pic, canister_id);

    let mut first_stakable = current_episode;
    while first_stakable % 3 != 2 {
        first_stakable += 1;
    }

    let absolute_episode = first_stakable + (relative_episode as u64 * 3);

    absolute_episode
}

pub fn get_episode_time_to_end(pic: &PocketIc, target_episode: u64) -> u64 {
    let target_episode_end_time = (target_episode + 1) * icp_canister_backend::EPISODE_DURATION;
    let current_time = pic.get_time().as_nanos_since_unix_epoch() / 1_000_000_000;
    target_episode_end_time - current_time
}

pub fn advance_time(pic: &PocketIc, duration_seconds: u64) {
    pic.advance_time(std::time::Duration::from_secs(duration_seconds));
    pic.tick();
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
            Principal::anonymous(),
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
        .update_call(canister_id, Principal::anonymous(), "reward_pool", encode_args(()).unwrap())
        .map_err(|e| format!("Failed to call reward_pool: {:?}", e))?;

    let result: Result<(), PoolError> = decode_one(&reward_result)
        .map_err(|e| format!("Failed to decode reward_pool result: {:?}", e))?;

    result.map_err(|e| format!("Reward pool failed: {:?}", e))
}
