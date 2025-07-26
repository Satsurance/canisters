use candid::{encode_args, CandidType, Deserialize, Nat, Principal};
use icp_canister_backend::{Account, DepositError, TransferArg, TransferError};
use pocket_ic::PocketIc;

#[derive(CandidType, Deserialize, Debug)]
pub enum TransferResult {
    Ok(Nat),
    Err(TransferError),
}
lazy_static::lazy_static! {
    pub static ref TRANSFER_FEE: Nat = Nat::from(10_000u64);
    pub static ref MINIMUM_DEPOSIT_AMOUNT: Nat = Nat::from(100_000u64);
}

pub fn create_deposit(
    pic: &PocketIc,
    canister_id: Principal,
    ledger_id: Principal,
    user: Principal,
    deposit_amount: Nat,
    timelock: u64,
) {
    let subaccount_result = pic
        .query_call(
            canister_id,
            user,
            "get_deposit_subaccount",
            encode_args((user, timelock)).unwrap(),
        )
        .expect("Failed to get deposit subaccount");
    let subaccount: [u8; 32] = candid::decode_one(&subaccount_result).unwrap();

    let transfer_args = TransferArg {
        from_subaccount: None,
        to: Account {
            owner: canister_id,
            subaccount: Some(subaccount.to_vec()),
        },
        amount: deposit_amount.clone(),
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
        "Transfer failed: {:?}",
        transfer_result
    );

    let deposit_result = pic
        .update_call(
            canister_id,
            user,
            "deposit",
            encode_args((user, timelock)).unwrap(),
        )
        .expect("Failed to call deposit");
    let result: Result<(), DepositError> = candid::decode_one(&deposit_result).unwrap();
    assert!(result.is_ok(), "Deposit failed: {:?}", result);
}
