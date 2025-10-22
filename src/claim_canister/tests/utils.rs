#![allow(dead_code)]
use candid::{encode_args, decode_one, Nat, Principal};
use commons::{LedgerCanisterClient, ledger::ApproveArgs};
use pool_canister::{TRANSFER_FEE, types::Account};
use pocket_ic::PocketIc;

pub fn get_stakable_episode(pic: &PocketIc, pool_canister: Principal, caller: Principal) -> u64 {
    let current_episode_bytes = pic
        .query_call(pool_canister, caller, "get_current_episode_id", encode_args(()) .unwrap())
        .unwrap();
    let mut current_episode: u64 = decode_one(&current_episode_bytes).unwrap();

    while current_episode % 3 != 2 {
        current_episode += 1;
    }

    current_episode
}


pub fn approve_deposit(
    ledger_client: &mut LedgerCanisterClient,
    caller: Principal,
    claim_canister: Principal,
    amount: Nat,
) {
    let claim_account = Account {
        owner: claim_canister,
        subaccount: None,
    };
    let approve_args = ApproveArgs {
        from_subaccount: None,
        spender: claim_account,
        amount: amount.clone() + TRANSFER_FEE.clone(),
        expected_allowance: None,
        expires_at: None,
        fee: Some(TRANSFER_FEE.clone()),
        memo: None,
        created_at_time: None,
    };
    ledger_client.connect(caller).icrc2_approve(approve_args);
}

