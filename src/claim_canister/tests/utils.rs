#![allow(dead_code)]
use candid::{decode_one, encode_args, Nat, Principal};
use commons::LedgerCanisterClient;
use pocket_ic::PocketIc;
use pool_canister::{
    types::{Account, TransferArg},
    TRANSFER_FEE,
};

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

pub fn transfer_to_deposit_subaccount(
    ledger_client: &mut LedgerCanisterClient,
    caller: Principal,
    claim_canister: Principal,
    subaccount: [u8; 32],
    amount: Nat,
) {
    let claim_account = Account {
        owner: claim_canister,
        subaccount: Some(subaccount.to_vec()),
    };
    let transfer_args = TransferArg {
        from_subaccount: None,
        to: claim_account,
        amount: amount.clone(),
        fee: Some(TRANSFER_FEE.clone()),
        memo: None,
        created_at_time: None,
    };
    ledger_client.connect(caller).icrc1_transfer(transfer_args);
}
