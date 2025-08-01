use candid::{encode_args, Nat, Principal};
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

pub fn create_episode_deposit(
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

pub fn create_deposit_and_advance_time(
    pic: &PocketIc,
    canister_id: Principal,
    ledger_id: Principal,
    user: Principal,
    amount: Nat,
) -> u64 {
    let current_episode_result = pic
        .query_call(
            canister_id,
            user,
            "get_current_episode_id",
            encode_args(()).unwrap(),
        )
        .expect("Failed to get current episode");
    let current_episode: u64 = candid::decode_one(&current_episode_result).unwrap();

    create_episode_deposit(pic, canister_id, ledger_id, user, amount, current_episode);

    let episode_duration_seconds = 91 * 24 * 60 * 60 / 3;
    pic.advance_time(std::time::Duration::from_secs(episode_duration_seconds + 1));
    pic.tick();

    current_episode
}
