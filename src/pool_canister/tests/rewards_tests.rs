use candid::{Nat, Principal};
use pool_canister::EPISODE_DURATION;
use commons::{PoolCanisterClient, LedgerCanisterClient, TRANSFER_FEE, ALLOWED_ERROR, advance_time, get_stakable_episode_with_client, get_episode_time_to_end, create_deposit, reward_pool, get_current_time, assert_with_error};
mod setup;
use setup::setup;

#[test]
fn test_reward_rate_increase_decrease_during_episodes() {
    let (pic, pool_canister, ledger_id) = setup();
    let mut pool_client = PoolCanisterClient::new(&pic, pool_canister);
    let mut ledger_client = LedgerCanisterClient::new(&pic, ledger_id);
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let reward_amount = Nat::from(10_000_000u64); // 0.1 BTC

    // Check initial reward rate (should be 0)
    let initial_reward_rate = pool_client.connect(user).get_pool_reward_rate();
    assert_eq!(
        initial_reward_rate,
        Nat::from(0u64),
        "Initial reward rate should be 0"
    );

    // Create a user deposit first to test reward distribution
    let deposit_amount = Nat::from(100_000_000u64); // 1 BTC
    let stakable_episode = get_stakable_episode_with_client(&pool_client, 7);
    create_deposit(&mut pool_client, &mut ledger_client, user, deposit_amount.clone(), stakable_episode)
        .expect("Deposit should succeed");

    let reward_time = get_current_time(&pic);
    reward_pool(&mut pool_client, &mut ledger_client, user, reward_amount.clone()).expect("Reward pool should succeed");

    // Get the reward rate after reward_pool
    let increased_reward_rate = pool_client.get_pool_reward_rate();
    // Calculate timing to advance to end of reward period
    let last_reward_episode =
        (reward_time + pool_canister::EPISODE_DURATION * 12) / pool_canister::EPISODE_DURATION;

    let reward_duration = (last_reward_episode + 1) * pool_canister::EPISODE_DURATION - reward_time;
    let actual_amount = reward_amount.clone();
    let expected_rate_increase = (actual_amount.clone() * pool_canister::PRECISION_SCALE.clone())
        / Nat::from(reward_duration);

    assert!(
        increased_reward_rate > initial_reward_rate,
        "Reward rate should be increased after reward_pool. Initial: {}, After: {}",
        initial_reward_rate,
        increased_reward_rate
    );

    assert_eq!(
        increased_reward_rate, expected_rate_increase,
        "Reward rate should equal expected increase: {} tokens per second",
        expected_rate_increase
    );
    let time_to_reach_decrease_episode =
        (last_reward_episode + 1) * pool_canister::EPISODE_DURATION;
    let additional_time_needed = time_to_reach_decrease_episode - reward_time;

    advance_time(&pic, additional_time_needed);

    // Check that reward rate dropped to 0
    let decreased_reward_rate = pool_client.get_pool_reward_rate();
    assert_eq!(decreased_reward_rate, Nat::from(0u64));

    // Get user rewards after rate drop
    let rewards_after_rate_drop = pool_client.get_deposits_rewards(vec![0u64]);

    // Advance more time and verify no additional rewards
    advance_time(&pic, EPISODE_DURATION * 2);
    let rewards_after_additional_time = pool_client.get_deposits_rewards(vec![0u64]);

    assert_eq!(rewards_after_rate_drop, rewards_after_additional_time);

    // expected rewards amount
    let expected_distributed_reward = (increased_reward_rate.clone()
        * Nat::from(additional_time_needed))
        / pool_canister::PRECISION_SCALE.clone();
    assert_with_error!(
        &rewards_after_rate_drop,
        &expected_distributed_reward,
        &ALLOWED_ERROR,
        "Expected rewards amount verification"
    );
}

#[test]
fn test_reward_distribution_middle_and_final() {
    let (pic, pool_canister, ledger_id) = setup();
    let mut pool_client = PoolCanisterClient::new(&pic, pool_canister);
    let mut ledger_client = LedgerCanisterClient::new(&pic, ledger_id);
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let deposit_amount = Nat::from(100_000_000u64); // 1 BTC
    let reward_amount = Nat::from(25_000_000u64); // 0.25 BTC

    let stakable_episode = get_stakable_episode_with_client(&pool_client, 7);
    create_deposit(&mut pool_client, &mut ledger_client, user, deposit_amount.clone(), stakable_episode)
        .expect("Deposit should succeed");

    let reward_time = get_current_time(&pic);

    reward_pool(&mut pool_client, &mut ledger_client, user, reward_amount.clone()).expect("Reward pool should succeed");

    let last_reward_episode = (reward_time + EPISODE_DURATION * 12) / EPISODE_DURATION;
    let exact_reward_duration = (last_reward_episode + 1) * EPISODE_DURATION - reward_time;

    //Middle  rewards distribution
    let half_duration = exact_reward_duration / 2;
    advance_time(&pic, half_duration);

    let middle_rewards = pool_client.connect(user).get_deposits_rewards(vec![0u64]);

    let expected_middle = reward_amount.clone() / Nat::from(2u64);

    assert_with_error!(
        &middle_rewards,
        &expected_middle,
        &ALLOWED_ERROR,
        "Middle reward distribution"
    );

    let remaining_duration = exact_reward_duration - half_duration;
    advance_time(&pic, remaining_duration);

    let final_rewards = pool_client.get_deposits_rewards(vec![0u64]);

    // final reward distribution
    let expected_final = reward_amount.clone();
    assert_with_error!(
        &final_rewards,
        &expected_final,
        &ALLOWED_ERROR,
        "Full reward distribution"
    );

    // User balance before first withdrawal
    let balance_before_first = ledger_client.icrc1_balance_of(pool_canister::Account {
        owner: user,
        subaccount: None,
    });

    let withdrawn_amount = pool_client
        .withdraw_rewards(vec![0u64])
        .expect("First withdrawal should succeed");
    assert_with_error!(
        &withdrawn_amount,
        &expected_final,
        &ALLOWED_ERROR,
        "First withdrawal amount"
    );

    // Balance after first withdrawal
    let balance_after_first = ledger_client.icrc1_balance_of(pool_canister::Account {
        owner: user,
        subaccount: None,
    });
    let expected_after_first =
        balance_before_first.clone() + (withdrawn_amount.clone() - TRANSFER_FEE.clone());
    assert_eq!(
        balance_after_first, expected_after_first,
        "User balance after first withdrawal should increase by net amount"
    );

    // double withdrawal doesn't work
    let second_amount = pool_client
        .withdraw_rewards(vec![0u64])
        .unwrap_or(Nat::from(0u64));
    assert_eq!(
        second_amount,
        Nat::from(0u64),
        "Second withdrawal should return 0 tokens"
    );

    // Balance unchanged after second withdrawal
    let balance_after_second = ledger_client.icrc1_balance_of(pool_canister::Account {
        owner: user,
        subaccount: None,
    });
    assert_eq!(
        balance_after_second, balance_after_first,
        "User balance should remain unchanged after second withdrawal"
    );
}

#[test]
fn test_multiple_users_different_deposits_proportional_rewards() {
    let (pic, pool_canister, ledger_id) = setup();
    let mut pool_client = PoolCanisterClient::new(&pic, pool_canister);
    let mut ledger_client = LedgerCanisterClient::new(&pic, ledger_id);
    let user_a = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap();
    let user_b = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();

    let deposit_amount_a = Nat::from(100_000_000u64); // 1 BTC
    let deposit_amount_b = Nat::from(200_000_000u64); // 2 BTC
    let reward_amount = Nat::from(300_000_000u64); // 3 BTC

    let stakable_episode = get_stakable_episode_with_client(&pool_client, 7);
    create_deposit(
        &mut pool_client,
        &mut ledger_client,
        user_a,
        deposit_amount_a.clone(),
        stakable_episode,
    )
        .expect("Deposit should succeed");
    create_deposit(
        &mut pool_client,
        &mut ledger_client,
        user_b,
        deposit_amount_b.clone(),
        stakable_episode,
    )
        .expect("Deposit should succeed");
    let reward_start_time = get_current_time(&pic);
    reward_pool(&mut pool_client, &mut ledger_client, user_a, reward_amount.clone()).expect("Reward pool should succeed");

    // Advance to middle of reward period
    let exact_reward_duration =
        ((reward_start_time + EPISODE_DURATION * 12) / EPISODE_DURATION + 1) * EPISODE_DURATION
            - reward_start_time;
    advance_time(&pic, exact_reward_duration / 2);

    let rewards_a = pool_client.connect(user_a).get_deposits_rewards(vec![0u64]);
    let rewards_b = pool_client.connect(user_b).get_deposits_rewards(vec![1u64]);

    // Get actual shares and calculate expected rewards
    let deposits_a = pool_client.connect(user_a).get_user_deposits(user_a);
    let deposits_b = pool_client.connect(user_b).get_user_deposits(user_b);

    let shares_a = &deposits_a[0].shares;
    let shares_b = &deposits_b[0].shares;
    let total_shares = shares_a.clone() + shares_b.clone();
    let half_rewards = reward_amount.clone() / Nat::from(2u64);

    let expected_a = (half_rewards.clone() * shares_a.clone()) / total_shares.clone();
    let expected_b = (half_rewards * shares_b.clone()) / total_shares;

    assert_with_error!(
        &rewards_a,
        &expected_a,
        &ALLOWED_ERROR,
        "User A proportional rewards"
    );
    assert_with_error!(
        &rewards_b,
        &expected_b,
        &ALLOWED_ERROR,
        "User B proportional rewards"
    );
}

#[test]
fn test_users_joining_different_times_fair_distribution() {
    let (pic, pool_canister, ledger_id) = setup();
    let mut pool_client = PoolCanisterClient::new(&pic, pool_canister);
    let mut ledger_client = LedgerCanisterClient::new(&pic, ledger_id);
    let user_early = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap();
    let user_late = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();

    let deposit_amount = Nat::from(100_000_000u64); // 1 BTC
    let reward_amount = Nat::from(200_000_000u64); // 2 BTC

    let stakable_episode = get_stakable_episode_with_client(&pool_client, 7);
    create_deposit(
        &mut pool_client,
        &mut ledger_client,
        user_early,
        deposit_amount.clone(),
        stakable_episode,
    )
        .expect("Deposit should succeed");
    reward_pool(&mut pool_client, &mut ledger_client, user_early, reward_amount.clone())
        .expect("Reward pool should succeed");

    let reward_time = get_current_time(&pic);
    let last_reward_episode = (reward_time + EPISODE_DURATION * 12) / EPISODE_DURATION;
    let exact_reward_duration = (last_reward_episode + 1) * EPISODE_DURATION - reward_time;

    advance_time(&pic, exact_reward_duration / 4);

    create_deposit(
        &mut pool_client,
        &mut ledger_client,
        user_late,
        deposit_amount.clone(),
        stakable_episode,
    )
        .expect("Deposit should succeed");
    advance_time(&pic, (exact_reward_duration * 3) / 4);

    let rewards_early = pool_client.connect(user_early).get_deposits_rewards(vec![0u64]);
    let rewards_late = pool_client.connect(user_late).get_deposits_rewards(vec![1u64]);

    // - First 1/4 of the window: only early user participates -> 100% weight
    // - Remaining 3/4 of the window: both users participate equally -> 50% each
    // Early user's share = 1/4 * 1.0 + 3/4 * 0.5 = 0.25 + 0.375 = 0.625 (625/1000)
    // Late user's share  = 0/4 * 1.0 + 3/4 * 0.5 = 0 + 0.375 = 0.375 (375/1000)
    let expected_early = (reward_amount.clone() * Nat::from(625u64)) / Nat::from(1000u64);

    let expected_late = (reward_amount.clone() * Nat::from(375u64)) / Nat::from(1000u64);

    assert_with_error!(
        &rewards_early,
        &expected_early,
        &ALLOWED_ERROR,
        "Early user fair rewards"
    );
    assert_with_error!(
        &rewards_late,
        &expected_late,
        &ALLOWED_ERROR,
        "Late user fair rewards"
    );
}

#[test]
fn test_reward_withdrawal_ownership_and_security() {
    let (pic, pool_canister, ledger_id) = setup();
    let mut pool_client = PoolCanisterClient::new(&pic, pool_canister);
    let mut ledger_client = LedgerCanisterClient::new(&pic, ledger_id);
    let user_a = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap();
    let user_b = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();
    let malicious_user = Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap();

    let deposit_amount = Nat::from(100_000_000u64); // 1 BTC
    let reward_amount = Nat::from(100_000_000u64); // 1 BTC
    let stakable_episode = get_stakable_episode_with_client(&pool_client, 7);
    create_deposit(
        &mut pool_client,
        &mut ledger_client,
        user_a,
        deposit_amount.clone(),
        stakable_episode,
    )
        .expect("Deposit should succeed");
    create_deposit(
        &mut pool_client,
        &mut ledger_client,
        user_b,
        deposit_amount.clone(),
        stakable_episode,
    )
        .expect("Deposit should succeed");
    reward_pool(&mut pool_client, &mut ledger_client, user_a, reward_amount.clone()).expect("Reward pool should succeed");

    let reward_time = get_current_time(&pic);
    let last_reward_episode = (reward_time + EPISODE_DURATION * 12) / EPISODE_DURATION;
    let exact_reward_duration = (last_reward_episode + 1) * EPISODE_DURATION - reward_time;
    advance_time(&pic, exact_reward_duration);

    let result = pool_client.connect(user_a).withdraw_rewards(vec![1u64]);
    assert!(
        matches!(result, Err(pool_canister::PoolError::NotOwner)),
        "Expected NotOwner error, got: {:?}",
        result
    );

    let result_2 = pool_client.connect(malicious_user).withdraw_rewards(vec![0u64]);
    assert!(
        matches!(result_2, Err(pool_canister::PoolError::NotOwner)),
        "Expected NotOwner error for malicious user, got: {:?}",
        result_2
    );

    let result = pool_client.connect(user_b).withdraw_rewards(Vec::<u64>::new());
    assert!(
        matches!(result, Err(pool_canister::PoolError::InsufficientBalance)),
        "Expected InsufficientBalance error for empty withdrawal, got: {:?}",
        result
    );

    let withdrawn_amount = pool_client
        .connect(user_a)
        .withdraw_rewards(vec![0u64])
        .expect("Valid user should be able to withdraw their own rewards");

    //user A gets half since there are 2 equal deposits
    let expected_withdrawal = reward_amount.clone() / Nat::from(2u64);
    assert_with_error!(
        &withdrawn_amount,
        &expected_withdrawal,
        &ALLOWED_ERROR,
        "Valid withdrawal amount"
    );
}


#[test]
fn test_multiple_reward_pool_additions_cumulative() {
    let (pic, pool_canister, ledger_id) = setup();
    let mut pool_client = PoolCanisterClient::new(&pic, pool_canister);
    let mut ledger_client = LedgerCanisterClient::new(&pic, ledger_id);
    let user = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap();

    let deposit_amount = Nat::from(100_000_000u64); // 1 BTC
    let first_reward = Nat::from(50_000_000u64); // 0.5 BTC
    let second_reward = Nat::from(30_000_000u64); // 0.3 BTC
    let third_reward = Nat::from(20_000_000u64); // 0.2 BTC
    let total_expected_rewards =
        first_reward.clone() + second_reward.clone() + third_reward.clone(); // 1 BTC total

    let stakable_episode = get_stakable_episode_with_client(&pool_client, 7);
    create_deposit(&mut pool_client, &mut ledger_client, user, deposit_amount.clone(), stakable_episode)
        .expect("Deposit should succeed");

    // Add all three rewards
    reward_pool(&mut pool_client, &mut ledger_client, user, first_reward.clone()).expect("First reward pool should succeed");

    advance_time(&pic, EPISODE_DURATION / 4);
    reward_pool(&mut pool_client, &mut ledger_client, user, second_reward.clone())
        .expect("Second reward pool should succeed");

    advance_time(&pic, EPISODE_DURATION / 4);
    let third_reward_time = get_current_time(&pic);
    reward_pool(&mut pool_client, &mut ledger_client, user, third_reward.clone()).expect("Third reward pool should succeed");

    let last_reward_window_end =
        ((third_reward_time + EPISODE_DURATION * 12) / EPISODE_DURATION + 1) * EPISODE_DURATION;
    let current_time = get_current_time(&pic);

    advance_time(&pic, last_reward_window_end - current_time);

    let final_rewards = pool_client.connect(user).get_deposits_rewards(vec![0u64]);

    assert_with_error!(
        &final_rewards,
        &total_expected_rewards,
        &ALLOWED_ERROR,
        "Multiple reward additions final total"
    );
}

#[test]
fn test_reward_distribution_between_large_and_small_deposits() {
    let (pic, pool_canister, ledger_id) = setup();
    let mut pool_client = PoolCanisterClient::new(&pic, pool_canister);
    let mut ledger_client = LedgerCanisterClient::new(&pic, ledger_id);
    let large_depositor = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap();
    let small_depositor = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();

    let large_deposit = Nat::from(100_000_000u64); // 1 BTC
    let small_deposit = pool_canister::MINIMUM_DEPOSIT_AMOUNT.clone() + Nat::from(49_000u64); // 0.0005 BTC - $50 USD at $100k/BTC
    let reward_amount = Nat::from(3_000u64); //   0.00003 BTC - $3 USD at $100k/BTC

    let episode = get_stakable_episode_with_client(&pool_client, 7);

    create_deposit(&mut pool_client, &mut ledger_client, large_depositor, large_deposit, episode)
        .expect("Deposit should succeed");
    create_deposit(&mut pool_client, &mut ledger_client, small_depositor, small_deposit, episode)
        .expect("Deposit should succeed");

    let reward_start_time = get_current_time(&pic);
    reward_pool(&mut pool_client, &mut ledger_client, large_depositor, reward_amount.clone())
        .expect("Reward pool should succeed");

    let last_reward_episode = (reward_start_time + EPISODE_DURATION * 12) / EPISODE_DURATION;
    let exact_reward_duration = (last_reward_episode + 1) * EPISODE_DURATION - reward_start_time;
    advance_time(&pic, exact_reward_duration);

    let large_depositor_rewards = pool_client
        .connect(large_depositor)
        .get_deposits_rewards(vec![0u64]);
    let small_depositor_rewards = pool_client
        .connect(small_depositor)
        .get_deposits_rewards(vec![1u64]);

    //Get actual shares to compute precise expected values
    let large_deposits = pool_client
        .connect(large_depositor)
        .get_user_deposits(large_depositor);
    let small_deposits = pool_client
        .connect(small_depositor)
        .get_user_deposits(small_depositor);

    let large_shares = &large_deposits[0].shares;
    let small_shares = &small_deposits[0].shares;
    let total_shares = large_shares.clone() + small_shares.clone();

    let expected_large = (reward_amount.clone() * large_shares.clone()) / total_shares.clone();
    let expected_small = (reward_amount.clone() * small_shares.clone()) / total_shares.clone();

    let one_sat_allowed_error = Nat::from(1u64);
    assert_with_error!(
        &large_depositor_rewards,
        &expected_large,
        &one_sat_allowed_error,
        "Large depositor rewards"
    );
    assert_with_error!(
        &small_depositor_rewards,
        &expected_small,
        &one_sat_allowed_error,
        "Small depositor rewards"
    );

    let total_distributed = large_depositor_rewards + small_depositor_rewards;
    assert_with_error!(
        &total_distributed,
        &reward_amount,
        &ALLOWED_ERROR,
        "Total distributed equals reward"
    );
}

#[test]
fn test_partial_reward_withdrawals_during_period() {
    let (pic, pool_canister, ledger_id) = setup();
    let mut pool_client = PoolCanisterClient::new(&pic, pool_canister);
    let mut ledger_client = LedgerCanisterClient::new(&pic, ledger_id);
    let user = Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap();

    let deposit_amount = Nat::from(100_000_000u64); // 1 BTC
    let reward_amount = Nat::from(200_000_000u64); // 2 BTC

    let stakable_episode = get_stakable_episode_with_client(&pool_client, 7);
    create_deposit(&mut pool_client, &mut ledger_client, user, deposit_amount.clone(), stakable_episode)
        .expect("Deposit should succeed");

    let reward_start_time = get_current_time(&pic);
    reward_pool(&mut pool_client, &mut ledger_client, user, reward_amount.clone()).expect("Reward pool should succeed");

    let last_reward_episode = (reward_start_time + EPISODE_DURATION * 12) / EPISODE_DURATION;
    let exact_reward_duration = (last_reward_episode + 1) * EPISODE_DURATION - reward_start_time;

    // Advance to 1/2 of reward period
    advance_time(&pic, exact_reward_duration / 2);

    // Withdraw rewards at 1/2 period
    let first_withdrawn = pool_client
        .connect(user)
        .withdraw_rewards(vec![0u64])
        .expect("First withdrawal should succeed");

    // Verify first withdrawal is approximately half of total rewards
    let expected_half = reward_amount.clone() / Nat::from(2u64);
    assert_with_error!(
        &first_withdrawn,
        &expected_half,
        &ALLOWED_ERROR,
        "First withdrawal at 1/2 period"
    );

    // Advance to end of reward period
    advance_time(&pic, exact_reward_duration / 2);
    // Withdraw remaining rewards at end
    let second_withdrawn = pool_client
        .withdraw_rewards(vec![0u64])
        .expect("Second withdrawal should succeed");

    // Verify second withdrawal is approximately the remaining half
    let expected_remaining = reward_amount.clone() - first_withdrawn.clone();
    assert_with_error!(
        &second_withdrawn,
        &expected_remaining,
        &ALLOWED_ERROR,
        "Second withdrawal at end"
    );

    // Verify total withdrawn equals total reward pool
    let total_withdrawn = first_withdrawn + second_withdrawn;
    assert_with_error!(
        &total_withdrawn,
        &reward_amount,
        &ALLOWED_ERROR,
        "Total withdrawn equals reward pool"
    );

    let final_rewards = pool_client
        .connect(Principal::anonymous())
        .get_deposits_rewards(vec![0u64]);

    assert_eq!(
        final_rewards,
        Nat::from(0u64),
        "No rewards should remain after both withdrawals"
    );
}

#[test]
fn test_slashing_during_reward_distribution() {
    let (pic, pool_canister, ledger_id) = setup();
    let mut pool_client = PoolCanisterClient::new(&pic, pool_canister);
    let mut ledger_client = LedgerCanisterClient::new(&pic, ledger_id);
    let user_a = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap();
    let user_b = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();
    let executor = user_a.clone();
    let slash_receiver = Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap();

    let deposit_amount = Nat::from(100_000_000u64); // 1 BTC each
    let reward_amount = Nat::from(200_000_000u64); // 2 BTC total rewards
    let slash_amount = Nat::from(50_000_000u64); // 0.5 BTC to be slashed

    // Create deposits for both users
    let stakable_episode = get_stakable_episode_with_client(&pool_client, 7);
    create_deposit(
        &mut pool_client,
        &mut ledger_client,
        user_a,
        deposit_amount.clone(),
        stakable_episode,
    )
        .expect("Deposit should succeed");
    create_deposit(
        &mut pool_client,
        &mut ledger_client,
        user_b,
        deposit_amount.clone(),
        stakable_episode,
    )
        .expect("Deposit should succeed");

    let reward_start_time = get_current_time(&pic);
    reward_pool(&mut pool_client, &mut ledger_client, user_a, reward_amount.clone()).expect("Reward pool should succeed");

    // Calculate reward distribution timing
    let last_reward_episode = (reward_start_time + EPISODE_DURATION * 12) / EPISODE_DURATION;
    let exact_reward_duration = (last_reward_episode + 1) * EPISODE_DURATION - reward_start_time;

    // Advance to middle of reward period to accumulate some rewards
    advance_time(&pic, exact_reward_duration / 2);

    // Check rewards before slashing
    let rewards_a_before_slash = pool_client.connect(user_a).get_deposits_rewards(vec![0u64]);
    let rewards_b_before_slash = pool_client.connect(user_b).get_deposits_rewards(vec![1u64]);

    // Perform slashing
    pool_client
        .connect(executor)
        .slash(slash_receiver, slash_amount.clone())
        .expect("Slashing should succeed");

    // Check rewards immediately after slashing - they should be the same
    let rewards_a_after_slash = pool_client.connect(user_a).get_deposits_rewards(vec![0u64]);
    let rewards_b_after_slash = pool_client.connect(user_b).get_deposits_rewards(vec![1u64]);

    assert_eq!(
        rewards_a_after_slash, rewards_a_before_slash,
        "User A rewards should be unchanged immediately after slashing"
    );
    assert_eq!(
        rewards_b_after_slash, rewards_b_before_slash,
        "User B rewards should be unchanged immediately after slashing"
    );

    // Advance to end of reward period
    advance_time(&pic, exact_reward_duration / 2);

    // Check final rewards - should equal expected total regardless of slashing
    let final_rewards_a = pool_client.connect(user_a).get_deposits_rewards(vec![0u64]);
    let final_rewards_b = pool_client.connect(user_b).get_deposits_rewards(vec![1u64]);

    let expected_reward_per_user = reward_amount.clone() / Nat::from(2u64);
    assert_with_error!(
        &final_rewards_a,
        &expected_reward_per_user,
        &ALLOWED_ERROR,
        "User A should get half of total rewards despite slashing"
    );
    assert_with_error!(
        &final_rewards_b,
        &expected_reward_per_user,
        &ALLOWED_ERROR,
        "User B should get half of total rewards despite slashing"
    );

    // Test that deposits can be withdrawn successfully after slashing
    let episode_end_time = get_episode_time_to_end(&pool_client, stakable_episode);
    advance_time(&pic, episode_end_time);

    // Get user balances before withdrawal to verify they receive their deposits back
    let user_a_account = pool_canister::Account {
        owner: user_a,
        subaccount: None,
    };
    let user_b_account = pool_canister::Account {
        owner: user_b,
        subaccount: None,
    };
    let balance_a_before = ledger_client
        .connect(user_a)
        .icrc1_balance_of(user_a_account.clone());
    let balance_b_before = ledger_client
        .connect(user_b)
        .icrc1_balance_of(user_b_account.clone());

    // Withdraw deposits
    pool_client
        .connect(user_a)
        .withdraw(0u64)
        .expect("User A should be able to withdraw deposit after slashing");
    pool_client
        .connect(user_b)
        .withdraw(1u64)
        .expect("User B should be able to withdraw deposit after slashing");

    // Verify users received their deposit amounts back (minus transfer fees)
    let balance_a_after = ledger_client.connect(user_a).icrc1_balance_of(user_a_account);
    let balance_b_after = ledger_client.connect(user_b).icrc1_balance_of(user_b_account);

    // Calculate expected balances (deposit amount minus transfer fees)
    let expected_balance_increase = deposit_amount.clone() - slash_amount.clone() / Nat::from(2u64)
        + expected_reward_per_user
        - TRANSFER_FEE.clone()
        - TRANSFER_FEE.clone();

    assert_with_error!(
        &(balance_a_after - balance_a_before.clone()),
        &expected_balance_increase,
        &ALLOWED_ERROR,
        "User A should receive deposit amount back minus transfer fee"
    );
    assert_with_error!(
        &(balance_b_after - balance_b_before.clone()),
        &expected_balance_increase,
        &ALLOWED_ERROR,
        "User B should receive deposit amount back minus transfer fee"
    );
}
