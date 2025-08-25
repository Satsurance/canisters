use candid::{decode_one, encode_args, Nat, Principal};
use icp_canister_backend::PoolState;

mod setup;
use setup::setup;
mod utils;
use utils::{
    advance_time, create_deposit, get_episode_time_to_end, get_stakable_episode, TRANSFER_FEE,
};
#[test]
fn test_timer_episode_processing_exact_reduction() {
    let (pic, canister_id, ledger_id) = setup();
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let deposit_amount_1 = Nat::from(100_000_000u64);
    let deposit_amount_2 = Nat::from(200_000_000u64);

    let first_episode = get_stakable_episode(&pic, canister_id, 0);
    let second_episode = get_stakable_episode(&pic, canister_id, 1);

    // Create first deposit in first stakable episode
    create_deposit(
        &pic,
        canister_id,
        ledger_id,
        user,
        deposit_amount_1.clone(),
        first_episode,
    );

    // Record pool state after first deposit
    let pool_state_after_first = pic
        .query_call(
            canister_id,
            user,
            "get_pool_state",
            encode_args(()).unwrap(),
        )
        .expect("Failed to get pool state");
    let pool_after_first: PoolState = decode_one(&pool_state_after_first).unwrap();

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
    create_deposit(
        &pic,
        canister_id,
        ledger_id,
        user,
        deposit_amount_2.clone(),
        second_episode,
    );

    // Record pool state after second deposit (both episodes should be active)
    let pool_state_after_second = pic
        .query_call(
            canister_id,
            user,
            "get_pool_state",
            encode_args(()).unwrap(),
        )
        .expect("Failed to get pool state");
    let pool_after_second: PoolState = decode_one(&pool_state_after_second).unwrap();

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
    let episode_1_before = pic
        .query_call(
            canister_id,
            user,
            "get_episode",
            encode_args((first_episode,)).unwrap(),
        )
        .expect("Failed to get episode 1");
    let episode_1_data: Option<icp_canister_backend::Episode> =
        decode_one(&episode_1_before).unwrap();
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
    let first_episode_time_to_end = get_episode_time_to_end(&pic, first_episode);
    advance_time(&pic, first_episode_time_to_end);

    // Verify first episode was processed
    let episode_1_after = pic
        .query_call(
            canister_id,
            user,
            "get_episode",
            encode_args((first_episode,)).unwrap(),
        )
        .expect("Failed to get episode 1 after processing");
    let episode_1_processed: Option<icp_canister_backend::Episode> =
        decode_one(&episode_1_after).unwrap();
    assert!(
        episode_1_processed.is_some(),
        "Episode 1 should still exist after processing"
    );
    let _episode_1_final = episode_1_processed.unwrap();

    // Only first episode expired, second is still active
    let pool_state_final = pic
        .query_call(
            canister_id,
            user,
            "get_pool_state",
            encode_args(()).unwrap(),
        )
        .expect("Failed to get final pool state");
    let pool_final: PoolState = decode_one(&pool_state_final).unwrap();

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
    let episode_2_check = pic
        .query_call(
            canister_id,
            user,
            "get_episode",
            encode_args((second_episode,)).unwrap(),
        )
        .expect("Failed to get episode 2");
    let episode_2_data: Option<icp_canister_backend::Episode> =
        decode_one(&episode_2_check).unwrap();
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
    let (pic, canister_id, _) = setup();

    // Test that stakable episodes follow the pattern (episode % 3 == 2)
    for relative_episode in 0u8..8u8 {
        let stakable_episode = get_stakable_episode(&pic, canister_id, relative_episode);

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
        get_stakable_episode(&pic, canister_id, 9u8);
    }));
    assert!(
        panic_result.is_err(),
        "Expected panic for relative episode 9"
    );
}
