use candid::{Nat, Principal};
mod utils;
use utils::{TRANSFER_FEE, create_deposit, get_stakable_episode, get_episode_time_to_end, advance_time, setup::setup};
#[test]
fn test_timer_episode_processing_exact_reduction() {
    let s = setup();
    let mut client = s.client();
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let deposit_amount_1 = Nat::from(100_000_000u64);
    let deposit_amount_2 = Nat::from(200_000_000u64);

    let first_episode = get_stakable_episode(&client, 0);
    let second_episode = get_stakable_episode(&client, 1);

    // Create first deposit in first stakable episode
    create_deposit(&mut client, user, deposit_amount_1.clone(), first_episode);

    // Record pool state after first deposit
    let pool_after_first = client.connect(user).get_pool_state();

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
    create_deposit(&mut client, user, deposit_amount_2.clone(), second_episode);

    // Record pool state after second deposit (both episodes should be active)
    let pool_after_second = client.get_pool_state();

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
    let episode_1_data = client.get_episode(first_episode);
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
    let first_episode_time_to_end = get_episode_time_to_end(&client, first_episode);
    advance_time(&client, first_episode_time_to_end);

    // Verify first episode was processed
    let episode_1_processed = client.get_episode(first_episode);
    assert!(
        episode_1_processed.is_some(),
        "Episode 1 should still exist after processing"
    );
    let _episode_1_final = episode_1_processed.unwrap();

    // Only first episode expired, second is still active
    let pool_final = client.get_pool_state();

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
    let episode_2_data = client.get_episode(second_episode);
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
    let s = setup();
    let client = s.client();

    // Test that stakable episodes follow the pattern (episode % 3 == 2)
    for relative_episode in 0u8..8u8 {
        let stakable_episode = get_stakable_episode(&client, relative_episode);

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
        get_stakable_episode(&client, 9u8);
    }));
    assert!(
        panic_result.is_err(),
        "Expected panic for relative episode 9"
    );
}
