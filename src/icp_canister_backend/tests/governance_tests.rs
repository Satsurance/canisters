use candid::{decode_one, encode_args, Nat, Principal};
use icp_canister_backend::{types::UserDepositInfo, Account, PoolError, PoolState};

mod setup;
use setup::setup;
mod utils;
use utils::{
    advance_time, create_deposit, get_episode_time_to_end, get_stakable_episode, TRANSFER_FEE,
};
#[test]
fn test_slash_function() {
    let (pic, canister_id, ledger_id) = setup();
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let executor = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap();
    let receiver = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();
    let deposit_amount_1 = Nat::from(300_000_000u64);
    let deposit_amount_2 = Nat::from(200_000_000u64);
    let slash_amount = Nat::from(100_000_000u64);

    let first_episode = get_stakable_episode(&pic, canister_id, 0);

    // Create two deposits
    create_deposit(
        &pic,
        canister_id,
        ledger_id,
        user,
        deposit_amount_1.clone(),
        first_episode,
    );

    let second_episode = get_stakable_episode(&pic, canister_id, 1);
    create_deposit(
        &pic,
        canister_id,
        ledger_id,
        user,
        deposit_amount_2.clone(),
        second_episode,
    );

    // Check user deposits before slash
    let deposits_before = pic
        .query_call(
            canister_id,
            user,
            "get_user_deposits",
            encode_args((user,)).unwrap(),
        )
        .expect("Failed to get user deposits");
    let user_deposits_before: Vec<UserDepositInfo> = decode_one(&deposits_before).unwrap();

    // Verify initial deposit values
    assert_eq!(user_deposits_before.len(), 2, "User should have 2 deposits");
    let expected_amount_1 = deposit_amount_1.clone() - TRANSFER_FEE.clone();
    let expected_amount_2 = deposit_amount_2.clone() - TRANSFER_FEE.clone();
    assert_eq!(
        user_deposits_before[0].amount, expected_amount_1,
        "First deposit should have correct initial amount"
    );
    assert_eq!(
        user_deposits_before[1].amount, expected_amount_2,
        "Second deposit should have correct initial amount"
    );

    // Check pool state before slash
    let pool_state_before = pic
        .query_call(
            canister_id,
            user,
            "get_pool_state",
            encode_args(()).unwrap(),
        )
        .expect("Failed to get pool state");
    let pool_before: PoolState = decode_one(&pool_state_before).unwrap();

    let total_assets_before = expected_amount_1.clone() + expected_amount_2.clone();
    assert_eq!(
        pool_before.total_assets, total_assets_before,
        "Pool should have correct total assets before slash"
    );

    // Execute slash
    let slash_result = pic
        .update_call(
            canister_id,
            executor,
            "slash",
            encode_args((receiver, slash_amount.clone())).unwrap(),
        )
        .expect("Failed to execute slash");
    let result: Result<(), PoolError> = decode_one(&slash_result).unwrap();
    assert!(
        matches!(result, Ok(_)),
        "Slash should succeed: {:?}",
        result
    );

    // Check user deposits after slash - values should be proportionally reduced
    let deposits_after = pic
        .query_call(
            canister_id,
            user,
            "get_user_deposits",
            encode_args((user,)).unwrap(),
        )
        .expect("Failed to get user deposits");
    let user_deposits_after: Vec<UserDepositInfo> = decode_one(&deposits_after).unwrap();

    // Calculate expected amounts based on proportional reduction
    let total_reduction = slash_amount.clone();
    let total_original = expected_amount_1.clone() + expected_amount_2.clone();

    // Calculate reduction for each deposit proportionally
    let reduction_1 = total_reduction.clone() * expected_amount_1.clone() / total_original.clone();
    let reduction_2 = total_reduction.clone() * expected_amount_2.clone() / total_original.clone();

    let expected_amount_after_1 = expected_amount_1.clone() - reduction_1;
    let expected_amount_after_2 = expected_amount_2.clone() - reduction_2;

    assert_eq!(
        user_deposits_after[0].amount, expected_amount_after_1,
        "First deposit should be reduced proportionally after slash"
    );
    assert_eq!(
        user_deposits_after[1].amount, expected_amount_after_2,
        "Second deposit should be reduced proportionally after slash"
    );

    // Check pool state after slash
    let pool_state_after = pic
        .query_call(
            canister_id,
            user,
            "get_pool_state",
            encode_args(()).unwrap(),
        )
        .expect("Failed to get pool state");
    let pool_after: PoolState = decode_one(&pool_state_after).unwrap();

    // Calculate actual reduction based on proportional slashing precision
    let reduction_1 =
        slash_amount.clone() * expected_amount_1.clone() / total_assets_before.clone();
    let reduction_2 =
        slash_amount.clone() * expected_amount_2.clone() / total_assets_before.clone();
    let actual_total_reduction = reduction_1.clone() + reduction_2.clone();
    let expected_assets = pool_before.total_assets.clone() - actual_total_reduction;
    assert_eq!(
        pool_after.total_assets, expected_assets,
        "Pool assets should be reduced by actual accumulated slash amount"
    );

    // Test withdrawal after slash - advance time first
    let first_episode_time_to_end = get_episode_time_to_end(&pic, first_episode);
    advance_time(&pic, first_episode_time_to_end);

    // Get user balance before withdrawal
    let user_account = Account {
        owner: user,
        subaccount: None,
    };
    let balance_before_withdraw = pic
        .query_call(
            ledger_id,
            user,
            "icrc1_balance_of",
            encode_args((user_account.clone(),)).unwrap(),
        )
        .expect("Failed to check user balance before withdrawal");
    let balance_before: Nat = decode_one(&balance_before_withdraw).unwrap();

    let withdraw_result = pic
        .update_call(canister_id, user, "withdraw", encode_args((0u64,)).unwrap())
        .expect("Failed to call withdraw");
    let withdraw_res: Result<(), PoolError> = decode_one(&withdraw_result).unwrap();
    assert!(
        matches!(withdraw_res, Ok(_)),
        "Withdrawal should succeed after slash"
    );

    // Check user balance after FIRST withdrawal only
    let balance_after_withdraw = pic
        .query_call(
            ledger_id,
            user,
            "icrc1_balance_of",
            encode_args((user_account.clone(),)).unwrap(),
        )
        .expect("Failed to check user balance after withdrawal");
    let balance_after: Nat = decode_one(&balance_after_withdraw).unwrap();

    // Calculate expected withdrawal amount (reduced by slash)
    let expected_withdrawal_amount = expected_amount_after_1.clone() - TRANSFER_FEE.clone();
    let expected_balance_after = balance_before.clone() + expected_withdrawal_amount.clone();

    assert_eq!(
        balance_after, expected_balance_after,
        "User should receive correct withdrawal amount after slash. Expected: {}, Got: {}",
        expected_balance_after, balance_after
    );

    // Verify that the second deposit is also possible to withdraw
    let second_episode_time_to_end = get_episode_time_to_end(&pic, second_episode);
    advance_time(&pic, second_episode_time_to_end);

    let withdraw_result = pic
        .update_call(canister_id, user, "withdraw", encode_args((1u64,)).unwrap())
        .expect("Failed to call withdraw");
    let withdraw_res_2: Result<(), PoolError> = decode_one(&withdraw_result).unwrap();
    assert!(
        matches!(withdraw_res_2, Ok(_)),
        "Second withdrawal should also succeed after slash"
    );

    // Verify receiver got the slashed tokens
    let receiver_account = Account {
        owner: receiver,
        subaccount: None,
    };
    let receiver_balance_result = pic
        .query_call(
            ledger_id,
            user,
            "icrc1_balance_of",
            encode_args((receiver_account,)).unwrap(),
        )
        .expect("Failed to check receiver balance");
    let receiver_balance: Nat = decode_one(&receiver_balance_result).unwrap();

    // Calculate actual accumulated slashed amount due to proportional precision
    let actual_accumulated_slashed = reduction_1.clone() + reduction_2.clone();
    let expected_received = actual_accumulated_slashed - TRANSFER_FEE.clone();
    
    let receiver_initial_balance = Nat::from(10_000_000_000u64);
    let expected_total_balance = receiver_initial_balance + expected_received;
    
    assert_eq!(
        receiver_balance, expected_total_balance,
        "Receiver should have received actual accumulated slashed tokens minus fees"
    );
}
