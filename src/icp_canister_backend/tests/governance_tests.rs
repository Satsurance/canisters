use candid::{Nat, Principal};
use icp_canister_backend::Account;
mod setup;
use setup::setup;
mod utils;
use utils::TRANSFER_FEE;
#[test]
fn test_slash_function() {
    let s = setup();
    let mut client = s.client();
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let executor = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap();
    let receiver = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();
    let deposit_amount_1 = Nat::from(300_000_000u64);
    let deposit_amount_2 = Nat::from(200_000_000u64);
    let slash_amount = Nat::from(100_000_000u64);

    let first_episode = client.get_stakable_episode(0);

    // Create two deposits
    client
        .connect(user)
        .create_deposit(user, deposit_amount_1.clone(), first_episode);

    let second_episode = client.get_stakable_episode(1);
    client.create_deposit(user, deposit_amount_2.clone(), second_episode);

    // Check user deposits before slash
    let user_deposits_before = client.connect(user).get_user_deposits(user);

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
    let pool_before = client.get_pool_state();

    let total_assets_before = expected_amount_1.clone() + expected_amount_2.clone();
    assert_eq!(
        pool_before.total_assets, total_assets_before,
        "Pool should have correct total assets before slash"
    );

    // Execute slash
    let result = client
        .connect(executor)
        .slash(receiver, slash_amount.clone());
    assert!(
        matches!(result, Ok(_)),
        "Slash should succeed: {:?}",
        result
    );

    // Check user deposits after slash - values should be proportionally reduced
    let user_deposits_after = client.connect(user).get_user_deposits(user);

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
    let pool_after = client.get_pool_state();

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
    let first_episode_time_to_end = client.get_episode_time_to_end(first_episode);
    client.advance_time(first_episode_time_to_end);

    // Get user balance before withdrawal
    let user_account = Account {
        owner: user,
        subaccount: None,
    };
    let balance_before = client.connect(user).icrc1_balance_of(user_account.clone());

    let withdraw_res = client.withdraw(0u64);
    assert!(
        matches!(withdraw_res, Ok(_)),
        "Withdrawal should succeed after slash"
    );

    // Check user balance after FIRST withdrawal only
    let balance_after = client.icrc1_balance_of(user_account.clone());

    // Calculate expected withdrawal amount (reduced by slash)
    let expected_withdrawal_amount = expected_amount_after_1.clone() - TRANSFER_FEE.clone();
    let expected_balance_after = balance_before.clone() + expected_withdrawal_amount.clone();

    assert_eq!(
        balance_after, expected_balance_after,
        "User should receive correct withdrawal amount after slash. Expected: {}, Got: {}",
        expected_balance_after, balance_after
    );

    // Verify that the second deposit is also possible to withdraw
    let second_episode_time_to_end = client.get_episode_time_to_end(second_episode);
    client.advance_time(second_episode_time_to_end);

    let withdraw_res_2 = client.withdraw(1u64);
    assert!(
        matches!(withdraw_res_2, Ok(_)),
        "Second withdrawal should also succeed after slash"
    );

    // Verify receiver got the slashed tokens
    let receiver_account = Account {
        owner: receiver,
        subaccount: None,
    };
    let receiver_balance = client.icrc1_balance_of(receiver_account);

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
