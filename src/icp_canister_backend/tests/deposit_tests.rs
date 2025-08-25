use candid::{decode_one, encode_args, Nat, Principal};
use icp_canister_backend::{types::UserDepositInfo, Account, Deposit, PoolError, PoolState};
use sha2::{Digest, Sha256};

mod setup;
use setup::setup;
mod utils;
use utils::{
    advance_time, create_deposit, get_current_episode, get_episode_time_to_end,
    get_stakable_episode, TRANSFER_FEE,
};

#[test]
fn test_get_deposit_subaccount() {
    let (pic, canister_id, _) = setup();
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let episode: u64 = 123456789;

    let result = pic
        .query_call(
            canister_id,
            user,
            "get_deposit_subaccount",
            encode_args((user.clone(), episode)).unwrap(),
        )
        .expect("Failed to call get_deposit_subaccount");

    let returned_subaccount: [u8; 32] = decode_one(&result).unwrap();

    // Expected subaccount calculation
    let mut hasher = Sha256::new();
    hasher.update(user.as_slice());
    hasher.update(episode.to_be_bytes());
    let expected_subaccount: [u8; 32] = hasher.finalize().into();

    assert_eq!(returned_subaccount, expected_subaccount);
}

#[test]
fn test_deposit_fails_without_transfer() {
    let (pic, canister_id, _ledger_id) = setup();
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();

    let current_episode = get_stakable_episode(&pic, canister_id, 0);

    // Directly call deposit without transferring tokens first
    let deposit_result = pic
        .update_call(
            canister_id,
            user,
            "deposit",
            encode_args((user, current_episode)).unwrap(),
        )
        .expect("Failed to call deposit");

    let result: Result<(), PoolError> = decode_one(&deposit_result).unwrap();
    assert!(
        matches!(result, Err(PoolError::InsufficientBalance)),
        "Expected InsufficientBalance error, got: {:?}",
        result
    );
}

#[test]
fn test_shares_calculation() {
    let (pic, canister_id, ledger_id) = setup();
    let user1 = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();

    let current_episode = get_stakable_episode(&pic, canister_id, 0);

    let deposit_amount = Nat::from(200_000_000u64);

    // First user deposits
    create_deposit(
        &pic,
        canister_id,
        ledger_id,
        user1,
        deposit_amount.clone(),
        current_episode,
    );

    let pool_state_result = pic
        .query_call(
            canister_id,
            user1,
            "get_pool_state",
            encode_args(()).unwrap(),
        )
        .expect("Failed to get pool state");
    let pool_state: PoolState = decode_one(&pool_state_result).unwrap();

    let expected_amount = deposit_amount.clone() - TRANSFER_FEE.clone();
    assert_eq!(
        pool_state.total_assets, expected_amount,
        "Pool should have correct assets"
    );
    assert_eq!(
        pool_state.total_shares, expected_amount,
        "Pool should have correct shares"
    );

    // Create a second deposit from the same user to test proportional shares
    let next_episode = get_stakable_episode(&pic, canister_id, 1);
    create_deposit(
        &pic,
        canister_id,
        ledger_id,
        user1,
        deposit_amount.clone(),
        next_episode,
    );

    let pool_state_after_second = pic
        .query_call(
            canister_id,
            user1,
            "get_pool_state",
            encode_args(()).unwrap(),
        )
        .expect("Failed to get pool state");
    let pool_state_after: PoolState = decode_one(&pool_state_after_second).unwrap();

    let expected_total_assets = (deposit_amount.clone() - TRANSFER_FEE.clone()) * 2u64;
    let expected_total_shares = (deposit_amount.clone() - TRANSFER_FEE.clone()) * 2u64;

    assert_eq!(
        pool_state_after.total_assets, expected_total_assets,
        "Pool should have doubled assets"
    );
    assert_eq!(
        pool_state_after.total_shares, expected_total_shares,
        "Pool should have doubled shares"
    );

    // Check both deposits have equal shares since they were equal amounts
    let user1_deposits = pic
        .query_call(
            canister_id,
            user1,
            "get_user_deposits",
            encode_args((user1,)).unwrap(),
        )
        .expect("Failed to get user1 deposits");
    let user1_deposits: Vec<UserDepositInfo> = decode_one(&user1_deposits).unwrap();

    assert_eq!(user1_deposits.len(), 2, "User should have 2 deposits");
    assert_eq!(
        user1_deposits[0].shares, user1_deposits[1].shares,
        "Both deposits should have equal shares"
    );
    assert_eq!(
        user1_deposits[0].amount, user1_deposits[1].amount,
        "Both deposits should have equal amount"
    );

    let expected_shares = deposit_amount.clone() - TRANSFER_FEE.clone();
    assert_eq!(
        user1_deposits[0].shares, expected_shares,
        "First deposit should have expected shares"
    );
    assert_eq!(
        user1_deposits[1].shares, expected_shares,
        "Second deposit should have expected shares"
    );
}

#[test]
fn test_deposit_fails_below_minimum_amount() {
    let (pic, canister_id, ledger_id) = setup();
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();

    let current_episode = get_stakable_episode(&pic, canister_id, 0);

    let small_deposit_amount = Nat::from(50_000u64);

    // Get subaccount
    let subaccount_result = pic
        .query_call(
            canister_id,
            user,
            "get_deposit_subaccount",
            encode_args((user, current_episode)).unwrap(),
        )
        .expect("Failed to get deposit subaccount");
    let subaccount: [u8; 32] = decode_one(&subaccount_result).unwrap();

    // Transfer small amount to subaccount
    let transfer_args = icp_canister_backend::TransferArg {
        from_subaccount: None,
        to: Account {
            owner: canister_id,
            subaccount: Some(subaccount.to_vec()),
        },
        amount: small_deposit_amount.clone(),
        fee: Some(utils::TRANSFER_FEE.clone()),
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
    let transfer_result: utils::TransferResult = decode_one(&transfer_result).unwrap();
    assert!(
        matches!(transfer_result, utils::TransferResult::Ok(_)),
        "Transfer should succeed"
    );

    // Try to create deposit - should fail
    let deposit_result = pic
        .update_call(
            canister_id,
            user,
            "deposit",
            encode_args((user, current_episode)).unwrap(),
        )
        .expect("Failed to call deposit");
    let result: Result<(), PoolError> = decode_one(&deposit_result).unwrap();
    assert!(
        matches!(result, Err(PoolError::InsufficientBalance)),
        "Expected InsufficientBalance error for deposit below minimum, got: {:?}",
        result
    );
}

#[test]
fn test_deposit_flow() {
    let (pic, canister_id, ledger_id) = setup();
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let deposit_amount = Nat::from(100_000_000u64);

    let current_episode = get_stakable_episode(&pic, canister_id, 0);

    create_deposit(
        &pic,
        canister_id,
        ledger_id,
        user,
        deposit_amount.clone(),
        current_episode,
    );

    // Verify the canister main account has the tokens
    let main_account = Account {
        owner: canister_id,
        subaccount: None,
    };

    let balance_check = pic
        .query_call(
            ledger_id,
            user,
            "icrc1_balance_of",
            encode_args((main_account,)).unwrap(),
        )
        .expect("Failed to check canister balance");

    let canister_balance: Nat = decode_one(&balance_check).unwrap();
    let expected_balance = deposit_amount.clone() - TRANSFER_FEE.clone();
    assert_eq!(
        canister_balance, expected_balance,
        "Canister should have received the exact expected tokens"
    );

    // Check pool state after deposit
    let pool_state_result = pic
        .query_call(
            canister_id,
            user,
            "get_pool_state",
            encode_args(()).unwrap(),
        )
        .expect("Failed to get pool state");
    let pool_state: PoolState = decode_one(&pool_state_result).unwrap();

    let expected_assets = deposit_amount.clone() - TRANSFER_FEE.clone();
    assert_eq!(
        pool_state.total_assets, expected_assets,
        "Pool should have correct total assets"
    );
    assert_eq!(
        pool_state.total_shares, expected_assets,
        "First deposit should have 1:1 share ratio"
    );
}

#[test]
fn test_successful_withdrawal() {
    let (pic, canister_id, ledger_id) = setup();
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let deposit_amount = Nat::from(100_000_000u64);

    // Check user's initial balance before deposit
    let user_account = Account {
        owner: user,
        subaccount: None,
    };
    let initial_balance_result = pic
        .query_call(
            ledger_id,
            user,
            "icrc1_balance_of",
            encode_args((user_account.clone(),)).unwrap(),
        )
        .expect("Failed to check user initial balance");
    let initial_balance: Nat = decode_one(&initial_balance_result).unwrap();

    // Create deposit and advance time to simulate finished episode
    let current_episode = get_stakable_episode(&pic, canister_id, 0);
    create_deposit(
        &pic,
        canister_id,
        ledger_id,
        user,
        deposit_amount.clone(),
        current_episode,
    );
    let current_episode_time_to_end = get_episode_time_to_end(&pic, current_episode);
    advance_time(&pic, current_episode_time_to_end);

    // Now withdraw
    let withdraw_result = pic
        .update_call(canister_id, user, "withdraw", encode_args((0u64,)).unwrap())
        .expect("Failed to call withdraw");
    let result: Result<(), PoolError> = decode_one(&withdraw_result).unwrap();
    assert!(matches!(result, Ok(_)), "Withdraw failed: {:?}", result);

    // Verify user received the tokens back
    let final_balance_result = pic
        .query_call(
            ledger_id,
            user,
            "icrc1_balance_of",
            encode_args((user_account,)).unwrap(),
        )
        .expect("Failed to check user final balance");
    let final_balance: Nat = decode_one(&final_balance_result).unwrap();

    let expected_balance = initial_balance.clone() - (TRANSFER_FEE.clone() * 3u64);
    assert_eq!(
        final_balance, expected_balance,
        "User should have received tokens back. Initial: {}, Final: {}, Expected: {}",
        initial_balance, final_balance, expected_balance
    );

    // Check pool state after withdrawal
    let pool_state_result = pic
        .query_call(
            canister_id,
            user,
            "get_pool_state",
            encode_args(()).unwrap(),
        )
        .expect("Failed to get pool state");
    let pool_state: PoolState = decode_one(&pool_state_result).unwrap();

    assert_eq!(
        pool_state.total_assets,
        Nat::from(0u64),
        "Pool should have no assets after withdrawal"
    );
    assert_eq!(
        pool_state.total_shares,
        Nat::from(0u64),
        "Pool should have no shares after withdrawal"
    );
}

#[test]
fn test_deposit_episode_validation() {
    let (pic, canister_id, ledger_id) = setup();
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let deposit_amount = Nat::from(100_000_000u64);

    let current_episode = get_current_episode(&pic, canister_id);

    // Test deposit in past episode (should fail - time validation)
    if current_episode > 0 {
        let past_episode = current_episode - 1;
        let subaccount_result = pic
            .query_call(
                canister_id,
                user,
                "get_deposit_subaccount",
                encode_args((user, past_episode)).unwrap(),
            )
            .expect("Failed to get deposit subaccount");
        let subaccount: [u8; 32] = decode_one(&subaccount_result).unwrap();

        let transfer_args = icp_canister_backend::TransferArg {
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

        pic.update_call(
            ledger_id,
            user,
            "icrc1_transfer",
            encode_args((transfer_args,)).unwrap(),
        )
        .expect("Failed to transfer tokens");

        let deposit_result = pic
            .update_call(
                canister_id,
                user,
                "deposit",
                encode_args((user, past_episode)).unwrap(),
            )
            .expect("Failed to call deposit");

        let result: Result<(), PoolError> = decode_one(&deposit_result).unwrap();
        assert!(
            matches!(result, Err(PoolError::EpisodeNotActive)),
            "Expected EpisodeNotActive error for past episode, got: {:?}",
            result
        );
    }

    // Test deposit with non-stakable episode (should fail - pattern validation)
    let first_stakable_episode = get_stakable_episode(&pic, canister_id, 0);
    let non_stakable_episode = first_stakable_episode + 1;

    let subaccount_result = pic
        .query_call(
            canister_id,
            user,
            "get_deposit_subaccount",
            encode_args((user, non_stakable_episode)).unwrap(),
        )
        .expect("Failed to get deposit subaccount");
    let subaccount: [u8; 32] = decode_one(&subaccount_result).unwrap();

    let transfer_args = icp_canister_backend::TransferArg {
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

    pic.update_call(
        ledger_id,
        user,
        "icrc1_transfer",
        encode_args((transfer_args,)).unwrap(),
    )
    .expect("Failed to transfer tokens");

    let deposit_result = pic
        .update_call(
            canister_id,
            user,
            "deposit",
            encode_args((user, non_stakable_episode)).unwrap(),
        )
        .expect("Failed to call deposit");

    let result: Result<(), PoolError> = decode_one(&deposit_result).unwrap();
    assert!(
        matches!(result, Err(PoolError::EpisodeNotStakable)),
        "Expected EpisodeNotStakable error for non-stakable episode, got: {:?}",
        result
    );

    // Test deposit in far future stakable episode (should fail - not yet active)
    let latest_stakable_episode = get_stakable_episode(&pic, canister_id, 7);
    let far_future_stakable_episode = latest_stakable_episode + 3;

    let subaccount_result = pic
        .query_call(
            canister_id,
            user,
            "get_deposit_subaccount",
            encode_args((user, far_future_stakable_episode)).unwrap(),
        )
        .expect("Failed to get deposit subaccount");
    let subaccount: [u8; 32] = decode_one(&subaccount_result).unwrap();

    let transfer_args = icp_canister_backend::TransferArg {
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

    pic.update_call(
        ledger_id,
        user,
        "icrc1_transfer",
        encode_args((transfer_args,)).unwrap(),
    )
    .expect("Failed to transfer tokens");

    let deposit_result = pic
        .update_call(
            canister_id,
            user,
            "deposit",
            encode_args((user, far_future_stakable_episode)).unwrap(),
        )
        .expect("Failed to call deposit");

    let result: Result<(), PoolError> = decode_one(&deposit_result).unwrap();
    assert!(
        matches!(result, Err(PoolError::EpisodeNotActive)),
        "Expected EpisodeNotActive error for far future stakable episode, got: {:?}",
        result
    );

    // Test deposit in valid stakable episode (should succeed)
    let current_episode = get_stakable_episode(&pic, canister_id, 7); // Last stakable episode within range
    create_deposit(
        &pic,
        canister_id,
        ledger_id,
        user,
        deposit_amount.clone(),
        current_episode,
    );

    // Verify the deposit was created successfully
    let user_deposits = pic
        .query_call(
            canister_id,
            user,
            "get_user_deposits",
            encode_args((user,)).unwrap(),
        )
        .expect("Failed to get user deposits");
    let user_deposits: Vec<UserDepositInfo> = decode_one(&user_deposits).unwrap();

    assert_eq!(user_deposits.len(), 1, "User should have 1 deposit");
    assert_eq!(
        user_deposits[0].episode, current_episode,
        "Deposit should be in the stakable episode"
    );
}

#[test]
fn test_withdraw_before_timelock() {
    let (pic, canister_id, ledger_id) = setup();
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let deposit_amount = Nat::from(100_000_000u64);

    let current_episode = get_stakable_episode(&pic, canister_id, 0);

    create_deposit(
        &pic,
        canister_id,
        ledger_id,
        user,
        deposit_amount.clone(),
        current_episode,
    );

    // Try to withdraw before episode ends
    let withdraw_result = pic
        .update_call(canister_id, user, "withdraw", encode_args((0u64,)).unwrap())
        .expect("Failed to call withdraw");
    let result: Result<(), PoolError> = decode_one(&withdraw_result).unwrap();
    assert!(
        matches!(result, Err(PoolError::TimelockNotExpired)),
        "Expected TimelockNotExpired error, got: {:?}",
        result
    );
}

#[test]
fn test_withdraw_invalid_deposit_id() {
    let (pic, canister_id, _ledger_id) = setup();
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    // Try to withdraw with invalid deposit id
    let withdraw_result = pic
        .update_call(
            canister_id,
            user,
            "withdraw",
            encode_args((999u64,)).unwrap(),
        )
        .expect("Failed to call withdraw");
    let result: Result<(), PoolError> = decode_one(&withdraw_result).unwrap();
    assert!(
        matches!(result, Err(PoolError::NoDeposit)),
        "Expected NoDeposit error, got: {:?}",
        result
    );
}

#[test]
fn test_user_deposit_tracking() {
    let (pic, canister_id, ledger_id) = setup();
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let deposit_amount = Nat::from(100_000_000u64);

    // Check initial user deposits (should be empty)
    let initial_deposits_result = pic
        .query_call(
            canister_id,
            user,
            "get_user_deposits",
            encode_args((user,)).unwrap(),
        )
        .expect("Failed to get user deposits");
    let initial_deposits: Vec<UserDepositInfo> = decode_one(&initial_deposits_result).unwrap();
    assert_eq!(
        initial_deposits.len(),
        0,
        "User should have no deposits initially"
    );

    let first_episode = get_stakable_episode(&pic, canister_id, 0);

    // Create first deposit
    create_deposit(
        &pic,
        canister_id,
        ledger_id,
        user,
        deposit_amount.clone(),
        first_episode,
    );

    // Check user deposits after first deposit
    let deposits_after_first_result = pic
        .query_call(
            canister_id,
            user,
            "get_user_deposits",
            encode_args((user,)).unwrap(),
        )
        .expect("Failed to get user deposits");
    let deposits_after_first: Vec<UserDepositInfo> =
        decode_one(&deposits_after_first_result).unwrap();
    assert_eq!(deposits_after_first.len(), 1, "User should have 1 deposit");
    assert_eq!(
        deposits_after_first[0].deposit_id, 0,
        "First deposit should have ID 0"
    );

    let expected_amount = deposit_amount.clone() - TRANSFER_FEE.clone();
    assert_eq!(
        deposits_after_first[0].shares, expected_amount,
        "First deposit should have correct shares"
    );
    assert_eq!(
        deposits_after_first[0].amount, expected_amount,
        "First deposit amount should equal shares initially"
    );
    assert_eq!(
        deposits_after_first[0].episode, first_episode,
        "First deposit should have correct episode"
    );
    // Create second deposit in next episode
    let second_episode = get_stakable_episode(&pic, canister_id, 1);
    create_deposit(
        &pic,
        canister_id,
        ledger_id,
        user,
        deposit_amount.clone(),
        second_episode,
    );

    // Check user deposits after second deposit
    let deposits_after_second_result = pic
        .query_call(
            canister_id,
            user,
            "get_user_deposits",
            encode_args((user,)).unwrap(),
        )
        .expect("Failed to get user deposits");
    let deposits_after_second: Vec<UserDepositInfo> =
        decode_one(&deposits_after_second_result).unwrap();
    assert_eq!(
        deposits_after_second.len(),
        2,
        "User should have 2 deposits"
    );
    assert_eq!(
        deposits_after_second[0].deposit_id, 0,
        "First deposit should have ID 0"
    );
    assert_eq!(
        deposits_after_second[1].deposit_id, 1,
        "Second deposit should have ID 1"
    );
    assert_eq!(
        deposits_after_second[1].shares, expected_amount,
        "Second deposit should have proportional shares"
    );

    let third_episode = get_stakable_episode(&pic, canister_id, 2);
    create_deposit(
        &pic,
        canister_id,
        ledger_id,
        user,
        deposit_amount.clone(),
        third_episode,
    );

    let third_episode_time_to_end = get_episode_time_to_end(&pic, third_episode);
    advance_time(&pic, third_episode_time_to_end);

    let withdraw_result = pic
        .update_call(canister_id, user, "withdraw", encode_args((2u64,)).unwrap())
        .expect("Failed to call withdraw");
    let result: Result<(), PoolError> = decode_one(&withdraw_result).unwrap();
    assert!(matches!(result, Ok(_)), "Withdraw failed: {:?}", result);

    // Check user deposits after withdrawal
    let deposits_after_withdraw_result = pic
        .query_call(
            canister_id,
            user,
            "get_user_deposits",
            encode_args((user,)).unwrap(),
        )
        .expect("Failed to get user deposits");
    let deposits_after_withdraw: Vec<UserDepositInfo> =
        decode_one(&deposits_after_withdraw_result).unwrap();
    assert_eq!(
        deposits_after_withdraw.len(),
        2,
        "User should have 2 deposits after withdrawal (2 remaining)"
    );
}

#[test]
fn test_withdraw_invalid_principal() {
    let (pic, canister_id, ledger_id) = setup();
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let other = Principal::from_text("aaaaa-aa").unwrap();
    let deposit_amount = Nat::from(100_000_000u64);

    let current_episode = get_stakable_episode(&pic, canister_id, 0);

    create_deposit(
        &pic,
        canister_id,
        ledger_id,
        user,
        deposit_amount.clone(),
        current_episode,
    );

    // Try to withdraw as other principal
    let withdraw_result = pic
        .update_call(
            canister_id,
            other,
            "withdraw",
            encode_args((0u64,)).unwrap(),
        )
        .expect("Failed to call withdraw");
    let result: Result<(), PoolError> = decode_one(&withdraw_result).unwrap();
    assert!(
        matches!(result, Err(PoolError::NotOwner)),
        "Expected NotOwner error, got: {:?}",
        result
    );
}

#[test]
fn test_get_deposit() {
    let (pic, canister_id, ledger_id) = setup();
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let deposit_amount = Nat::from(100_000_000u64);

    // Try to get non-existent deposit
    let get_deposit_result = pic
        .query_call(
            canister_id,
            user,
            "get_deposit",
            encode_args((999u64,)).unwrap(),
        )
        .expect("Failed to call get_deposit");
    let non_existent_deposit: Option<Deposit> = decode_one(&get_deposit_result).unwrap();
    assert!(
        non_existent_deposit.is_none(),
        "Non-existent deposit should return None"
    );

    let current_episode = get_stakable_episode(&pic, canister_id, 0);

    // Create a deposit
    create_deposit(
        &pic,
        canister_id,
        ledger_id,
        user,
        deposit_amount.clone(),
        current_episode,
    );

    // Get the created deposit
    let get_deposit_result = pic
        .query_call(
            canister_id,
            user,
            "get_deposit",
            encode_args((0u64,)).unwrap(),
        )
        .expect("Failed to call get_deposit");
    let deposit: Option<Deposit> = decode_one(&get_deposit_result).unwrap();

    assert!(deposit.is_some(), "Deposit should exist");
    let deposit = deposit.unwrap();

    let expected_amount = deposit_amount.clone() - TRANSFER_FEE.clone();
    assert_eq!(
        deposit.shares, expected_amount,
        "Deposit should have correct shares"
    );
    assert_eq!(
        deposit.episode, current_episode,
        "Deposit should have correct episode"
    );
}
