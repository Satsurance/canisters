use candid::{decode_one, encode_args, Nat, Principal};
use icp_canister_backend::EPISODE_DURATION;

mod setup;
use setup::setup;
mod utils;
use utils::{advance_time, create_deposit, get_stakable_episode, reward_pool, ALLOWED_ERROR};

#[test]
fn test_reward_rate_increase_decrease_during_episodes() {
    let (pic, canister_id, ledger_id) = setup();
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let reward_amount = Nat::from(365_000_000u64);

    // Check initial reward rate (should be 0)
    let initial_reward_rate_result = pic
        .query_call(
            canister_id,
            user,
            "get_pool_reward_rate",
            encode_args(()).unwrap(),
        )
        .expect("Failed to get initial pool reward rate");
    let initial_reward_rate: Nat = decode_one(&initial_reward_rate_result).unwrap();
    assert_eq!(
        initial_reward_rate,
        Nat::from(0u64),
        "Initial reward rate should be 0"
    );

    let reward_time = pic.get_time().as_nanos_since_unix_epoch() / 1_000_000_000;
    reward_pool(&pic, canister_id, ledger_id, user, reward_amount.clone())
        .expect("Reward pool should succeed");

    // Check reward rate after reward_pool (should be increased)
    let increased_reward_rate_result = pic
        .query_call(
            canister_id,
            user,
            "get_pool_reward_rate",
            encode_args(()).unwrap(),
        )
        .expect("Failed to get pool reward rate after reward_pool");
    let increased_reward_rate: Nat = decode_one(&increased_reward_rate_result).unwrap();
    assert!(
        increased_reward_rate > initial_reward_rate,
        "Reward rate should be increased after reward_pool. Initial: {}, After: {}",
        initial_reward_rate,
        increased_reward_rate
    );

    let last_reward_episode = (reward_time + icp_canister_backend::EPISODE_DURATION * 12)
        / icp_canister_backend::EPISODE_DURATION;
    let reward_duration =
        (last_reward_episode + 1) * icp_canister_backend::EPISODE_DURATION - reward_time;
    let actual_amount = reward_amount.clone();
    let expected_rate_increase = (actual_amount.clone()
        * icp_canister_backend::PRECISION_SCALE.clone())
        / Nat::from(reward_duration);

    assert_eq!(
        increased_reward_rate, expected_rate_increase,
        "Reward rate should equal expected increase: {} tokens per second",
        expected_rate_increase
    );

    let target_episode_for_decrease = last_reward_episode + 1;
    let time_to_reach_decrease_episode =
        (target_episode_for_decrease + 1) * icp_canister_backend::EPISODE_DURATION;
    let additional_time_needed = time_to_reach_decrease_episode - reward_time;

    advance_time(&pic, additional_time_needed + 1);

    // Check reward rate after episode processing (should be decreased)
    let decreased_reward_rate_result = pic
        .query_call(
            canister_id,
            user,
            "get_pool_reward_rate",
            encode_args(()).unwrap(),
        )
        .expect("Failed to get pool reward rate after episode processing");
    let decreased_reward_rate: Nat = decode_one(&decreased_reward_rate_result).unwrap();

    // Should be exactly 0 since we decrease by the same amount we increased
    assert_eq!(
        decreased_reward_rate, Nat::from(0u64),
        "Reward rate should be back to 0 after processing episode with reward decrease. Final rate: {}",
        decreased_reward_rate
    );
}

#[test]
fn test_reward_distribution_middle_and_final() {
    let (pic, canister_id, ledger_id) = setup();
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let deposit_amount = Nat::from(100_000_000u64);
    let reward_amount = Nat::from(50_000_000u64);

    let stakable_episode = get_stakable_episode(&pic, canister_id, 7);
    create_deposit(
        &pic,
        canister_id,
        ledger_id,
        user,
        deposit_amount.clone(),
        stakable_episode,
    );

    let reward_time = pic.get_time().as_nanos_since_unix_epoch() / 1_000_000_000;

    reward_pool(&pic, canister_id, ledger_id, user, reward_amount.clone())
        .expect("Reward pool should succeed");

    let last_reward_episode = (reward_time + EPISODE_DURATION * 12) / EPISODE_DURATION;
    let exact_reward_duration = (last_reward_episode + 1) * EPISODE_DURATION - reward_time;

    //Middle  rewards distribution
    let half_duration = exact_reward_duration / 2;
    advance_time(&pic, half_duration);

    let middle_rewards = pic
        .query_call(
            canister_id,
            user,
            "get_deposits_rewards",
            encode_args((vec![0u64],)).unwrap(),
        )
        .map(|r| decode_one::<Nat>(&r).unwrap())
        .expect("Failed to get middle rewards");

    let expected_middle = reward_amount.clone() / Nat::from(2u64);

    assert_with_error!(
        &middle_rewards,
        &expected_middle,
        &ALLOWED_ERROR,
        "Middle reward distribution"
    );

    let remaining_duration = exact_reward_duration - half_duration;
    advance_time(&pic, remaining_duration);

    let final_rewards = pic
        .query_call(
            canister_id,
            user,
            "get_deposits_rewards",
            encode_args((vec![0u64],)).unwrap(),
        )
        .map(|r| decode_one::<Nat>(&r).unwrap())
        .expect("Failed to get final rewards");

    // final reward distribution
    let expected_final = reward_amount.clone();
    assert_with_error!(
        &final_rewards,
        &expected_final,
        &ALLOWED_ERROR,
        "Full reward distribution"
    );
}
