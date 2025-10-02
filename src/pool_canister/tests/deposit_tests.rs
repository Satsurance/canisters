use candid::{Nat, Principal};
use pool_canister::{Account, PoolError};
use sha2::{Digest, Sha256};
use commons::{PoolCanisterClient, LedgerCanisterClient, TRANSFER_FEE, ALLOWED_ERROR, advance_time, get_stakable_episode_with_client, get_episode_time_to_end, create_deposit, reward_pool, assert_with_error};
use commons::clients::ledger::TransferResult;
mod setup;
use setup::setup;

#[test]
fn test_get_deposit_subaccount() {
    let (pic, pool_canister, _ledger_id) = setup();
    let mut pool_client = PoolCanisterClient::new(&pic, pool_canister);
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let episode: u64 = 123456789;

    let returned_subaccount = pool_client.connect(user).get_deposit_subaccount(user, episode);

    // Expected subaccount calculation
    let mut hasher = Sha256::new();
    hasher.update(user.as_slice());
    hasher.update(episode.to_be_bytes());
    let expected_subaccount: [u8; 32] = hasher.finalize().into();

    assert_eq!(returned_subaccount, expected_subaccount);
}

#[test]
fn test_deposit_fails_without_transfer() {
    let (pic, pool_canister, _ledger_id) = setup();
    let mut pool_client = PoolCanisterClient::new(&pic, pool_canister);
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();

    let current_episode = get_stakable_episode_with_client(&pool_client, 0);

    // Directly call deposit without transferring tokens first
    let result = pool_client.connect(user).deposit(user, current_episode);
    assert!(
        matches!(result, Err(PoolError::InsufficientBalance)),
        "Expected InsufficientBalance error, got: {:?}",
        result
    );
}


#[test]
fn test_shares_calculation() {
    let (pic, pool_canister, ledger_id) = setup();
    let mut pool_client = PoolCanisterClient::new(&pic, pool_canister);
    let mut ledger_client = LedgerCanisterClient::new(&pic, ledger_id);
    let user1 = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();

    let current_episode = get_stakable_episode_with_client(&pool_client, 0);

    let deposit_amount = Nat::from(200_000_000u64);

    // First user deposits
    create_deposit(&mut pool_client, &mut ledger_client, user1, deposit_amount.clone(), current_episode)
        .expect("First deposit should succeed");
    let pool_state = pool_client.connect(user1).get_pool_state();

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
    let next_episode = get_stakable_episode_with_client(&pool_client, 1);
    create_deposit(&mut pool_client, &mut ledger_client, user1, deposit_amount.clone(), next_episode)
        .expect("Second deposit should succeed");

    let pool_state_after = pool_client.get_pool_state();

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
    let user1_deposits = pool_client.get_user_deposits(user1);

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
    let (pic, pool_canister, ledger_id) = setup();
    let mut pool_client = PoolCanisterClient::new(&pic, pool_canister);
    let mut ledger_client = LedgerCanisterClient::new(&pic, ledger_id);
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();

    let current_episode = get_stakable_episode_with_client(&pool_client, 0);

    let small_deposit_amount = Nat::from(500u64);

    // Get subaccount
    let subaccount = pool_client
        .connect(user)
        .get_deposit_subaccount(user, current_episode);

    // Transfer small amount to subaccount
    let transfer_args = pool_canister::TransferArg {
        from_subaccount: None,
        to: Account {
            owner: pool_canister,
            subaccount: Some(subaccount.to_vec()),
        },
        amount: small_deposit_amount.clone(),
        fee: Some(TRANSFER_FEE.clone()),
        memo: None,
        created_at_time: None,
    };
    let transfer_result = ledger_client.connect(user).icrc1_transfer(transfer_args);
    assert!(
        matches!(transfer_result, TransferResult::Ok(_)),
        "Transfer should succeed"
    );

    // Try to create deposit - should fail
    let result = pool_client.deposit(user, current_episode);
    assert!(
        matches!(result, Err(PoolError::InsufficientBalance)),
        "Expected InsufficientBalance error for deposit below minimum, got: {:?}",
        result
    );
}

#[test]
fn test_deposit_flow() {
    let (pic, pool_canister, ledger_id) = setup();
    let mut pool_client = PoolCanisterClient::new(&pic, pool_canister);
    let mut ledger_client = LedgerCanisterClient::new(&pic, ledger_id);
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let deposit_amount = Nat::from(100_000_000u64);

    let current_episode = get_stakable_episode_with_client(&pool_client, 0);

    create_deposit(&mut pool_client, &mut ledger_client, user, deposit_amount.clone(), current_episode)
        .expect("Deposit should succeed");

    // Verify the canister main account has the tokens
    let main_account = Account {
        owner: pool_canister,
        subaccount: None,
    };

    let canister_balance = ledger_client.connect(user).icrc1_balance_of(main_account);
    let expected_balance = deposit_amount.clone() - TRANSFER_FEE.clone();
    assert_eq!(
        canister_balance, expected_balance,
        "Canister should have received the exact expected tokens"
    );

    // Check pool state after deposit
    let pool_state = pool_client.get_pool_state();

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
    let (pic, pool_canister, ledger_id) = setup();
    let mut pool_client = PoolCanisterClient::new(&pic, pool_canister);
    let mut ledger_client = LedgerCanisterClient::new(&pic, ledger_id);
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let deposit_amount = Nat::from(100_000_000u64);

    // Check user's initial balance before deposit
    let user_account = Account {
        owner: user,
        subaccount: None,
    };
    let initial_balance = ledger_client.connect(user).icrc1_balance_of(user_account.clone());

    // Create deposit and advance time to simulate finished episode
    let current_episode = get_stakable_episode_with_client(&pool_client, 0);
    create_deposit(&mut pool_client, &mut ledger_client, user, deposit_amount.clone(), current_episode)
        .expect("Deposit should succeed");
    let current_episode_time_to_end = get_episode_time_to_end(&pool_client, current_episode);
    advance_time(&pic, current_episode_time_to_end);

    // Now withdraw
    let result = pool_client.withdraw(0u64);
    assert!(matches!(result, Ok(_)), "Withdraw failed: {:?}", result);

    // Verify user received the tokens back
    let final_balance = ledger_client.icrc1_balance_of(user_account);

    let expected_balance = initial_balance.clone() - (TRANSFER_FEE.clone() * 3u64);
    assert_eq!(
        final_balance, expected_balance,
        "User should have received tokens back. Initial: {}, Final: {}, Expected: {}",
        initial_balance, final_balance, expected_balance
    );

    // Check pool state after withdrawal
    let pool_state = pool_client.get_pool_state();

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
    let (pic, pool_canister, ledger_id) = setup();
    let mut pool_client = PoolCanisterClient::new(&pic, pool_canister);
    let mut ledger_client = LedgerCanisterClient::new(&pic, ledger_id);
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let deposit_amount = Nat::from(100_000_000u64);

    let current_episode = pool_client.get_current_episode_id();

    // Test deposit in past episode (should fail - time validation)
    if current_episode > 0 {
        let past_episode = current_episode - 1;
        let subaccount = pool_client
            .connect(user)
            .get_deposit_subaccount(user, past_episode);

        let transfer_args = pool_canister::TransferArg {
            from_subaccount: None,
            to: Account {
                owner: pool_canister,
                subaccount: Some(subaccount.to_vec()),
            },
            amount: deposit_amount.clone(),
            fee: Some(TRANSFER_FEE.clone()),
            memo: None,
            created_at_time: None,
        };

        ledger_client.connect(user).icrc1_transfer(transfer_args);

        let result = pool_client.deposit(user, past_episode);
        assert!(
            matches!(result, Err(PoolError::EpisodeNotActive)),
            "Expected EpisodeNotActive error for past episode, got: {:?}",
            result
        );
    }

    // Test deposit with non-stakable episode (should fail - pattern validation)
    let first_stakable_episode = get_stakable_episode_with_client(&pool_client, 0);
    let non_stakable_episode = first_stakable_episode + 1;

    let subaccount = pool_client.get_deposit_subaccount(user, non_stakable_episode);

    let transfer_args = pool_canister::TransferArg {
        from_subaccount: None,
        to: Account {
            owner: pool_canister,
            subaccount: Some(subaccount.to_vec()),
        },
        amount: deposit_amount.clone(),
        fee: Some(TRANSFER_FEE.clone()),
        memo: None,
        created_at_time: None,
    };

    ledger_client.connect(user).icrc1_transfer(transfer_args);

    let result = pool_client.deposit(user, non_stakable_episode);
    assert!(
        matches!(result, Err(PoolError::EpisodeNotStakable)),
        "Expected EpisodeNotStakable error for non-stakable episode, got: {:?}",
        result
    );

    // Test deposit in far future stakable episode (should fail - not yet active)
    let latest_stakable_episode = get_stakable_episode_with_client(&pool_client, 7);
    let far_future_stakable_episode = latest_stakable_episode + 3;

    let subaccount = pool_client.get_deposit_subaccount(user, far_future_stakable_episode);

    let transfer_args = pool_canister::TransferArg {
        from_subaccount: None,
        to: Account {
            owner: pool_canister,
            subaccount: Some(subaccount.to_vec()),
        },
        amount: deposit_amount.clone(),
        fee: Some(TRANSFER_FEE.clone()),
        memo: None,
        created_at_time: None,
    };

    ledger_client.connect(user).icrc1_transfer(transfer_args);

    let result = pool_client.deposit(user, far_future_stakable_episode);
    assert!(
        matches!(result, Err(PoolError::EpisodeNotActive)),
        "Expected EpisodeNotActive error for far future stakable episode, got: {:?}",
        result
    );

    // Test deposit in valid stakable episode (should succeed)
    let current_episode = get_stakable_episode_with_client(&pool_client, 7); // Last stakable episode within range
    create_deposit(&mut pool_client, &mut ledger_client, user, deposit_amount.clone(), current_episode)
        .expect("Deposit should succeed");

    // Verify the deposit was created successfully
    let user_deposits = pool_client.get_user_deposits(user);

    assert_eq!(user_deposits.len(), 1, "User should have 1 deposit");
    assert_eq!(
        user_deposits[0].episode, current_episode,
        "Deposit should be in the stakable episode"
    );
}

#[test]
fn test_withdraw_before_timelock() {
    let (pic, pool_canister, ledger_id) = setup();
    let mut pool_client = PoolCanisterClient::new(&pic, pool_canister);
    let mut ledger_client = LedgerCanisterClient::new(&pic, ledger_id);
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let deposit_amount = Nat::from(100_000_000u64);

    let current_episode = get_stakable_episode_with_client(&pool_client, 0);

    create_deposit(&mut pool_client, &mut ledger_client, user, deposit_amount.clone(), current_episode)
        .expect("Deposit should succeed");

    // Try to withdraw before episode ends
    let result = pool_client.connect(user).withdraw(0u64);
    assert!(
        matches!(result, Err(PoolError::TimelockNotExpired)),
        "Expected TimelockNotExpired error, got: {:?}",
        result
    );
}

#[test]
fn test_withdraw_invalid_deposit_id() {
    let (pic, pool_canister, _ledger_id) = setup();
    let mut pool_client = PoolCanisterClient::new(&pic, pool_canister);
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    // Try to withdraw with invalid deposit id
    let result = pool_client.connect(user).withdraw(999u64);
    assert!(
        matches!(result, Err(PoolError::NoDeposit)),
        "Expected NoDeposit error, got: {:?}",
        result
    );
}

#[test]
fn test_user_deposit_tracking() {
    let (pic, pool_canister, ledger_id) = setup();
    let mut pool_client = PoolCanisterClient::new(&pic, pool_canister);
    let mut ledger_client = LedgerCanisterClient::new(&pic, ledger_id);
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let deposit_amount = Nat::from(100_000_000u64);

    // Check initial user deposits (should be empty)
    let initial_deposits = pool_client.connect(user).get_user_deposits(user);
    assert_eq!(
        initial_deposits.len(),
        0,
        "User should have no deposits initially"
    );

    let first_episode = get_stakable_episode_with_client(&pool_client, 0);

    // Create first deposit
    create_deposit(&mut pool_client, &mut ledger_client, user, deposit_amount.clone(), first_episode)
        .expect("Deposit should succeed");

    // Check user deposits after first deposit
    let deposits_after_first = pool_client.get_user_deposits(user);
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
    let second_episode = get_stakable_episode_with_client(&pool_client, 1);
    create_deposit(&mut pool_client, &mut ledger_client, user, deposit_amount.clone(), second_episode)
        .expect("Deposit should succeed");

    // Check user deposits after second deposit
    let deposits_after_second = pool_client.get_user_deposits(user);
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

    let third_episode = get_stakable_episode_with_client(&pool_client, 2);
    create_deposit(&mut pool_client, &mut ledger_client, user, deposit_amount.clone(), third_episode)
        .expect("Deposit should succeed");

    let third_episode_time_to_end = get_episode_time_to_end(&pool_client, third_episode);
    advance_time(&pic, third_episode_time_to_end);

    let result = pool_client.withdraw(2u64);
    assert!(matches!(result, Ok(_)), "Withdraw failed: {:?}", result);

    // Check user deposits after withdrawal
    let deposits_after_withdraw = pool_client.get_user_deposits(user);
    assert_eq!(
        deposits_after_withdraw.len(),
        2,
        "User should have 2 deposits after withdrawal (2 remaining)"
    );
}


#[test]
fn test_withdraw_invalid_principal() {
    let (pic, pool_canister, ledger_id) = setup();
    let mut pool_client = PoolCanisterClient::new(&pic, pool_canister);
    let mut ledger_client = LedgerCanisterClient::new(&pic, ledger_id);
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let other = Principal::from_text("aaaaa-aa").unwrap();
    let deposit_amount = Nat::from(100_000_000u64);

    let current_episode = get_stakable_episode_with_client(&pool_client, 0);

    create_deposit(&mut pool_client, &mut ledger_client, user, deposit_amount.clone(), current_episode)
        .expect("Deposit should succeed");

    // Try to withdraw as other principal
    let result = pool_client.connect(other).withdraw(0u64);
    assert!(
        matches!(result, Err(PoolError::NotOwner)),
        "Expected NotOwner error, got: {:?}",
        result
    );
}


#[test]
fn test_get_deposit() {
    let (pic, pool_canister, ledger_id) = setup();
    let mut pool_client = PoolCanisterClient::new(&pic, pool_canister);
    let mut ledger_client = LedgerCanisterClient::new(&pic, ledger_id);
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let deposit_amount = Nat::from(100_000_000u64);

    // Try to get non-existent deposit
    let non_existent_deposit = pool_client.connect(user).get_deposit(999u64);
    assert!(
        non_existent_deposit.is_none(),
        "Non-existent deposit should return None"
    );

    let current_episode = get_stakable_episode_with_client(&pool_client, 0);

    // Create a deposit
    create_deposit(&mut pool_client, &mut ledger_client, user, deposit_amount.clone(), current_episode)
        .expect("Deposit should succeed");
    // Get the created deposit
    let deposit = pool_client.get_deposit(0u64);

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

#[test]
fn test_withdraw_automatically_collects_rewards() {
    let (pic, pool_canister, ledger_id) = setup();
    let mut pool_client = PoolCanisterClient::new(&pic, pool_canister);
    let mut ledger_client = LedgerCanisterClient::new(&pic, ledger_id);
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let deposit_amount = Nat::from(100_000_000u64); // 1 BTC
    let reward_amount = Nat::from(50_000_000u64); // 0.5 BTC in rewards

    // Create a deposit
    let current_episode = get_stakable_episode_with_client(&pool_client, 0);
    create_deposit(&mut pool_client, &mut ledger_client, user, deposit_amount.clone(), current_episode)
        .expect("Deposit should succeed");

    // Add significant rewards to the pool
    reward_pool(&mut pool_client, &mut ledger_client, user, reward_amount.clone())
        .expect("Reward pool should succeed");

     // Advance time to allow some rewards to accumulate
    let time_to_advance = pool_canister::EPISODE_DURATION / 4;
    advance_time(&pic, time_to_advance);
    pool_client.connect(user).update_episodes_state();

    // Get user's balance before withdrawal
    let user_account = pool_canister::Account {
        owner: user,
        subaccount: None,
    };
    let balance_before = ledger_client.connect(user).icrc1_balance_of(user_account.clone());

    // Advance time past the episode to allow withdrawal
    let time_to_end = get_episode_time_to_end(&pool_client, current_episode);
    advance_time(&pic, time_to_end + 1);

    // Update episodes state to ensure rewards are properly calculated
    pool_client.connect(user).update_episodes_state();

    // Get the final pending rewards
    let final_pending_rewards = pool_client.get_deposits_rewards(vec![0u64]);

    // Withdraw the deposit
    let withdraw_result = pool_client.connect(user).withdraw(0u64);
    assert!(withdraw_result.is_ok(), "Withdraw should succeed: {:?}", withdraw_result);

    // Get user's balance after withdrawal
    let balance_after = ledger_client.connect(user).icrc1_balance_of(user_account);
    let actual_received = balance_after.clone() - balance_before.clone();
    let deposit_net_amount = deposit_amount.clone() - TRANSFER_FEE.clone();
    let expected_total = deposit_net_amount.clone() + final_pending_rewards.clone() - TRANSFER_FEE.clone();

    // Verify the user received both deposit amount and rewards
    assert_with_error!(
        &actual_received,
        &expected_total,
        &ALLOWED_ERROR,
        "User should receive deposit amount plus rewards. Expected deposit: {}, expected rewards: {}, total expected: {}, actual: {}"
    );

    // Verify the deposit is removed
    let deposit_after = pool_client.get_deposit(0u64);
    assert!(deposit_after.is_none(), "Deposit should be removed after withdrawal");

    // Verify no more pending rewards
    let remaining_rewards = pool_client.get_deposits_rewards(vec![0u64]);
    assert_eq!(remaining_rewards, Nat::from(0u64), "Should have no pending rewards after withdrawal");
}
