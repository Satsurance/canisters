use candid::{Nat, Principal};
use commons::{PoolCanisterClient, LedgerCanisterClient, TRANSFER_FEE, advance_time, get_stakable_episode_with_client, get_episode_time_to_end, create_deposit};
mod setup;
use setup::setup;
#[test]
fn test_timer_episode_processing_exact_reduction() {
    let (pic, pool_canister, ledger_id) = setup();
    let mut pool_client = PoolCanisterClient::new(&pic, pool_canister);
    let mut ledger_client = LedgerCanisterClient::new(&pic, ledger_id);
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let deposit_amount_1 = Nat::from(100_000_000u64);
    let deposit_amount_2 = Nat::from(200_000_000u64);

    let first_episode = get_stakable_episode_with_client(&pool_client, 0);
    let second_episode = get_stakable_episode_with_client(&pool_client, 1);

    // Create first deposit in first stakable episode
    create_deposit(&mut pool_client, &mut ledger_client, user, deposit_amount_1.clone(), first_episode)
        .expect("First deposit should succeed");

    // Record pool state after first deposit
    let pool_after_first = pool_client.connect(user).get_pool_state();

    let expected_amount_1 = deposit_amount_1.clone() - TRANSFER_FEE.clone();
    assert_eq!(
        pool_after_first.total_assets, expected_amount_1,
        "Pool should have first deposit assets"
    );
    assert_eq!(
        pool_after_first.total_shares, expected_amount_1,
        "Pool should have first deposit shares"
    );

    // Create second deposit immediately in next stakable episode (before advancing time)
    create_deposit(&mut pool_client, &mut ledger_client, user, deposit_amount_2.clone(), second_episode)
        .expect("Second deposit should succeed");

    // Record pool state after second deposit (both episodes should be active)
    let pool_after_second = pool_client.get_pool_state();

    let expected_amount_2 = deposit_amount_2.clone() - TRANSFER_FEE.clone();
    let expected_total_assets = expected_amount_1.clone() + expected_amount_2.clone();
    let expected_total_shares = expected_amount_1.clone() + expected_amount_2.clone();

    assert_eq!(
        pool_after_second.total_assets, expected_total_assets,
        "Pool should have both deposits assets"
    );
    assert_eq!(
        pool_after_second.total_shares, expected_total_shares,
        "Pool should have both deposits shares"
    );

    // Get episode data before processing
    let episode_1_data = pool_client.get_episode(first_episode);
    assert!(episode_1_data.is_some(), "Episode 1 should exist");
    let episode_1 = episode_1_data.unwrap();

    assert_eq!(
        episode_1.episode_shares, expected_amount_1,
        "Episode 1 should have correct shares"
    );
    assert_eq!(
        episode_1.assets_staked, expected_amount_1,
        "Episode 1 should have correct assets"
    );

    // Advance time to make ONLY first stakable episode finish
    let first_episode_time_to_end = get_episode_time_to_end(&pool_client, first_episode);
    advance_time(&pic, first_episode_time_to_end);

    // Verify first episode was processed
    let episode_1_processed = pool_client.get_episode(first_episode);
    assert!(
        episode_1_processed.is_some(),
        "Episode 1 should still exist after processing"
    );
    let _episode_1_final = episode_1_processed.unwrap();

    // Only first episode expired, second is still active
    let pool_final = pool_client.get_pool_state();

    // Total should now be: (episode1 + episode2) - episode1 = episode2 only
    assert_eq!(
        pool_final.total_assets, expected_amount_2,
        "Pool assets should be reduced by exact episode 1 amount: {} - {} = {}",
        expected_total_assets, expected_amount_1, expected_amount_2
    );
    assert_eq!(
        pool_final.total_shares, expected_amount_2,
        "Pool shares should be reduced by exact episode 1 amount: {} - {} = {}",
        expected_total_shares, expected_amount_1, expected_amount_2
    );

    // Verify second episode is still active and unprocessed
    let episode_2_data = pool_client.get_episode(second_episode);
    assert!(episode_2_data.is_some(), "Episode 2 should exist");
    let episode_2 = episode_2_data.unwrap();

    assert_eq!(
        episode_2.episode_shares, expected_amount_2,
        "Episode 2 should have correct shares"
    );
    assert_eq!(
        episode_2.assets_staked, expected_amount_2,
        "Episode 2 should have correct assets"
    );
}

#[test]
fn test_stakable_episode_functionality() {
    let (pic, pool_canister, _ledger_id) = setup();
    let pool_client = PoolCanisterClient::new(&pic, pool_canister);

    // Test that stakable episodes follow the pattern (episode % 3 == 2)
    for relative_episode in 0u8..8u8 {
        let stakable_episode = get_stakable_episode_with_client(&pool_client, relative_episode);

        // Verify that the returned episode follows the stakable pattern
        assert_eq!(
            stakable_episode % 3,
            2,
            "Stakable episode {} should end in 2 when divided by 3",
            stakable_episode
        );
        println!(
            "Relative episode {} maps to absolute episode {}",
            relative_episode, stakable_episode
        );
    }

    // Test that relative episode 9 should fail (out of range)
    let panic_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        get_stakable_episode_with_client(&pool_client, 9u8);
    }));
    assert!(
        panic_result.is_err(),
        "Expected panic for relative episode 9"
    );
}
