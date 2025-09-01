use candid::{decode_one, encode_args, Nat, Principal};
use icp_canister_backend::EPISODE_DURATION;

mod setup;
use setup::setup;
mod utils;
use utils::{
    advance_time, create_deposit, get_stakable_episode, reward_pool, ALLOWED_ERROR
};


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

    let last_reward_episode = (reward_time + icp_canister_backend::EPISODE_DURATION * 12) / icp_canister_backend::EPISODE_DURATION;
    let reward_duration = (last_reward_episode + 1) * icp_canister_backend::EPISODE_DURATION - reward_time;
    let actual_amount = reward_amount.clone();
    let expected_rate_increase = (actual_amount.clone() * icp_canister_backend::PRECISION_SCALE.clone()) / Nat::from(reward_duration);

    assert_eq!(
        increased_reward_rate, expected_rate_increase,
        "Reward rate should equal expected increase: {} tokens per second",
        expected_rate_increase
    );

    let target_episode_for_decrease = last_reward_episode + 1;
    let time_to_reach_decrease_episode = (target_episode_for_decrease + 1) * icp_canister_backend::EPISODE_DURATION;
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
    create_deposit(&pic, canister_id, ledger_id, user, deposit_amount.clone(), stakable_episode);
    
    let reward_time = pic.get_time().as_nanos_since_unix_epoch() / 1_000_000_000;
    
    reward_pool(&pic, canister_id, ledger_id, user, reward_amount.clone())
        .expect("Reward pool should succeed");

    let last_reward_episode = (reward_time + EPISODE_DURATION * 12) / EPISODE_DURATION;
    let exact_reward_duration = (last_reward_episode + 1) * EPISODE_DURATION - reward_time;
    
      //Middle  rewards distribution
    let half_duration = exact_reward_duration / 2;
    advance_time(&pic, half_duration);
    
    let middle_rewards = pic
        .query_call(canister_id, user, "get_deposits_rewards", encode_args((vec![0u64],)).unwrap())
        .map(|r| decode_one::<Nat>(&r).unwrap())
        .expect("Failed to get middle rewards");
    
    
    let expected_middle = reward_amount.clone() / Nat::from(2u64);
   
    assert_with_error!(&middle_rewards, &expected_middle, &ALLOWED_ERROR, "Middle reward distribution");
    
    
    let remaining_duration = exact_reward_duration - half_duration;
    advance_time(&pic, remaining_duration);
    
    let final_rewards = pic
        .query_call(canister_id, user, "get_deposits_rewards", encode_args((vec![0u64],)).unwrap())
        .map(|r| decode_one::<Nat>(&r).unwrap())
        .expect("Failed to get final rewards");
    
    // final reward distribution
    let expected_final = reward_amount.clone();
    assert_with_error!(&final_rewards, &expected_final, &ALLOWED_ERROR, "Full reward distribution");
}

#[test]
fn test_multiple_users_different_deposits_proportional_rewards() {
    let (pic, canister_id, ledger_id) = setup();
    let user_a = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap();
    let user_b = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();
    
    let deposit_amount_a = Nat::from(100_000_000u64); 
    let deposit_amount_b = Nat::from(200_000_000u64); 
    let reward_amount = Nat::from(300_000_000u64);  
    
    let stakable_episode = get_stakable_episode(&pic, canister_id, 7);
    
    create_deposit(&pic, canister_id, ledger_id, user_a, deposit_amount_a.clone(), stakable_episode);
    create_deposit(&pic, canister_id, ledger_id, user_b, deposit_amount_b.clone(), stakable_episode);

    reward_pool(&pic, canister_id, ledger_id, user_a, reward_amount.clone())
        .expect("Reward pool should succeed");
    
    let reward_time = pic.get_time().as_nanos_since_unix_epoch() / 1_000_000_000;
    let last_reward_episode = (reward_time + EPISODE_DURATION * 12) / EPISODE_DURATION;
    let exact_reward_duration = (last_reward_episode + 1) * EPISODE_DURATION - reward_time;
    
    // Advance to middle of reward period
    advance_time(&pic, exact_reward_duration / 2);
    
    let rewards_a = pic
        .query_call(canister_id, user_a, "get_deposits_rewards", encode_args((vec![0u64],)).unwrap())
        .map(|r| decode_one::<Nat>(&r).unwrap())
        .expect("Failed to get user A rewards");
    
    let rewards_b = pic
        .query_call(canister_id, user_b, "get_deposits_rewards", encode_args((vec![1u64],)).unwrap())
        .map(|r| decode_one::<Nat>(&r).unwrap())
        .expect("Failed to get user B rewards");
    
    // User B should get exactly 2x the rewards of User A 
    let expected_rewards_a = reward_amount.clone() / Nat::from(6u64); 
    let expected_rewards_b = reward_amount.clone() / Nat::from(3u64); 
    
    let tolerance_a = expected_rewards_a.clone() / Nat::from(10000u64);
    let tolerance_b = expected_rewards_b.clone() / Nat::from(10000u64);
    assert_with_error!(&rewards_a, &expected_rewards_a, &tolerance_a, "User A proportional rewards");
    assert_with_error!(&rewards_b, &expected_rewards_b, &tolerance_b, "User B proportional rewards");
    
}

#[test]
fn test_users_joining_different_times_fair_distribution() {
    let (pic, canister_id, ledger_id) = setup();
    let user_early = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap();
    let user_late = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();
    
    let deposit_amount = Nat::from(100_000_000u64);
    let reward_amount = Nat::from(200_000_000u64);
    
    let stakable_episode = get_stakable_episode(&pic, canister_id, 7);
    
    create_deposit(&pic, canister_id, ledger_id, user_early, deposit_amount.clone(), stakable_episode);
    
    reward_pool(&pic, canister_id, ledger_id, user_early, reward_amount.clone())
        .expect("Reward pool should succeed");
    
    let reward_time = pic.get_time().as_nanos_since_unix_epoch() / 1_000_000_000;
    let last_reward_episode = (reward_time + EPISODE_DURATION * 12) / EPISODE_DURATION;
    let exact_reward_duration = (last_reward_episode + 1) * EPISODE_DURATION - reward_time;
    
    advance_time(&pic, exact_reward_duration / 4);
    
    create_deposit(&pic, canister_id, ledger_id, user_late, deposit_amount.clone(), stakable_episode);
    
    advance_time(&pic, (exact_reward_duration * 3) / 4);
    
    let rewards_early = pic
        .query_call(canister_id, user_early, "get_deposits_rewards", encode_args((vec![0u64],)).unwrap())
        .map(|r| decode_one::<Nat>(&r).unwrap())
        .expect("Failed to get early user rewards");
    
    let rewards_late = pic
        .query_call(canister_id, user_late, "get_deposits_rewards", encode_args((vec![1u64],)).unwrap())
        .map(|r| decode_one::<Nat>(&r).unwrap())
        .expect("Failed to get late user rewards");
    
    let expected_early = (reward_amount.clone() * Nat::from(625u64)) / Nat::from(1000u64);
    
    let expected_late = (reward_amount.clone() * Nat::from(375u64)) / Nat::from(1000u64);
    
    assert_with_error!(&rewards_early, &expected_early, &ALLOWED_ERROR, "Early user fair rewards");
    assert_with_error!(&rewards_late, &expected_late, &ALLOWED_ERROR, "Late user fair rewards");
    
}
#[test]
fn test_reward_withdrawal_ownership_and_security() {
    let (pic, canister_id, ledger_id) = setup();
    let user_a = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap();
    let user_b = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();
    let malicious_user = Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap();
    
    let deposit_amount = Nat::from(100_000_000u64);
    let reward_amount = Nat::from(100_000_000u64);
    
    let stakable_episode = get_stakable_episode(&pic, canister_id, 7);
    
    create_deposit(&pic, canister_id, ledger_id, user_a, deposit_amount.clone(), stakable_episode);
    create_deposit(&pic, canister_id, ledger_id, user_b, deposit_amount.clone(), stakable_episode);
    
    reward_pool(&pic, canister_id, ledger_id, user_a, reward_amount.clone())
        .expect("Reward pool should succeed");
      
    let reward_time = pic.get_time().as_nanos_since_unix_epoch() / 1_000_000_000;
    let last_reward_episode = (reward_time + EPISODE_DURATION * 12) / EPISODE_DURATION;
    let exact_reward_duration = (last_reward_episode + 1) * EPISODE_DURATION - reward_time;
    advance_time(&pic, exact_reward_duration);
    
    let malicious_withdrawal = pic
        .update_call(
            canister_id,
            user_a,
            "withdraw_rewards", 
            encode_args((vec![1u64],)).unwrap() 
        );
    
    let malicious_result = malicious_withdrawal
        .map(|response| decode_one::<Result<Nat, icp_canister_backend::PoolError>>(&response).unwrap())
        .unwrap_or(Err(icp_canister_backend::PoolError::NotOwner));
    
    let is_not_owner_error = matches!(malicious_result, Err(icp_canister_backend::PoolError::NotOwner));
    assert!(is_not_owner_error, "Expected NotOwner error, got: {:?}", malicious_result);
    
    let malicious_withdrawal_2 = pic
        .update_call(
            canister_id,
            malicious_user,
            "withdraw_rewards",
            encode_args((vec![0u64],)).unwrap()
        );
    
    let malicious_result_2 = malicious_withdrawal_2
        .map(|response| decode_one::<Result<Nat, icp_canister_backend::PoolError>>(&response).unwrap())
        .unwrap_or(Err(icp_canister_backend::PoolError::NotOwner));
    
    
    let is_not_owner_error_2 = matches!(malicious_result_2, Err(icp_canister_backend::PoolError::NotOwner));
    assert!(is_not_owner_error_2, "Expected NotOwner error for malicious user, got: {:?}", malicious_result_2);
    
    let empty_withdrawal = pic
        .update_call(
            canister_id,
            user_b,
            "withdraw_rewards",
            encode_args((Vec::<u64>::new(),)).unwrap()
        )
        .expect("Empty withdrawal call should not fail at protocol level");
    
    let result: Result<Nat, icp_canister_backend::PoolError> = decode_one(&empty_withdrawal).unwrap();
    let is_insufficient_balance = matches!(result, Err(icp_canister_backend::PoolError::InsufficientBalance));
    assert!(is_insufficient_balance, "Expected InsufficientBalance error for empty withdrawal, got: {:?}", result);
    
    let valid_withdrawal = pic
        .update_call(
            canister_id,
            user_a,
            "withdraw_rewards",
            encode_args((vec![0u64],)).unwrap()
        )
        .expect("Valid withdrawal should succeed");
    
    let result: Result<Nat, icp_canister_backend::PoolError> = decode_one(&valid_withdrawal).unwrap();
    let withdrawn_amount = result.expect("Valid user should be able to withdraw their own rewards");
    
    //  expected withdrawal amount (user A gets half since there are 2 equal deposits)
    let expected_withdrawal = reward_amount.clone() / Nat::from(2u64); 
    assert_with_error!(&withdrawn_amount, &expected_withdrawal, &ALLOWED_ERROR, "Valid withdrawal amount");
}

#[test] 
fn test_multiple_reward_pool_additions_cumulative() {
    let (pic, canister_id, ledger_id) = setup();
    let user = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap();
    
    let deposit_amount = Nat::from(100_000_000u64);
    let first_reward = Nat::from(50_000_000u64);
    let second_reward = Nat::from(30_000_000u64);
    let third_reward = Nat::from(20_000_000u64);
    let total_expected_rewards = first_reward.clone() + second_reward.clone() + third_reward.clone();
    
    let stakable_episode = get_stakable_episode(&pic, canister_id, 7);
    
    create_deposit(&pic, canister_id, ledger_id, user, deposit_amount.clone(), stakable_episode);
    
    // First reward addition
    reward_pool(&pic, canister_id, ledger_id, user, first_reward.clone())
        .expect("First reward pool should succeed");
    
    advance_time(&pic, EPISODE_DURATION / 4);
    // Second reward addition
    reward_pool(&pic, canister_id, ledger_id, user, second_reward.clone())
        .expect("Second reward pool should succeed");

    advance_time(&pic, EPISODE_DURATION / 4);
    // Third reward addition
    reward_pool(&pic, canister_id, ledger_id, user, third_reward.clone())
        .expect("Third reward pool should succeed");
    
    let current_reward_rate = pic
        .query_call(canister_id, user, "get_pool_reward_rate", encode_args(()).unwrap())
        .map(|r| decode_one::<Nat>(&r).unwrap())
        .expect("Failed to get reward rate");
    
    let current_time = pic.get_time().as_nanos_since_unix_epoch() / 1_000_000_000;
    let last_reward_episode = (current_time + EPISODE_DURATION * 12) / EPISODE_DURATION;
    let reward_duration = (last_reward_episode + 1) * EPISODE_DURATION - current_time;
    let expected_rate = (total_expected_rewards.clone() * icp_canister_backend::PRECISION_SCALE.clone()) / Nat::from(reward_duration);
    assert_with_error!(&current_reward_rate, &expected_rate, &(expected_rate.clone() / Nat::from(10u64)), "Accumulated reward rate calculation");
    
    let current_time = pic.get_time().as_nanos_since_unix_epoch() / 1_000_000_000;
    let future_episode = (current_time + EPISODE_DURATION * 15) / EPISODE_DURATION;
    let time_to_future = (future_episode + 1) * EPISODE_DURATION - current_time;
    advance_time(&pic, time_to_future);
    
    let final_rewards = pic
        .query_call(canister_id, user, "get_deposits_rewards", encode_args((vec![0u64],)).unwrap())
        .map(|r| decode_one::<Nat>(&r).unwrap())
        .expect("Failed to get final rewards");
    
    // Verify final rewards 
    let tolerance = total_expected_rewards.clone() / Nat::from(10u64); 
    assert_with_error!(&final_rewards, &total_expected_rewards, &tolerance, "Multiple reward additions final total");
}

#[test]
fn test_episode_transition_reward_accuracy() {
    let (pic, canister_id, ledger_id) = setup();
    let user = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap();
    
    let deposit_amount = Nat::from(100_000_000u64);
    let reward_amount = Nat::from(200_000_000u64);
    
    let stakable_episode = get_stakable_episode(&pic, canister_id, 7);
    
    create_deposit(&pic, canister_id, ledger_id, user, deposit_amount.clone(), stakable_episode);
    
    reward_pool(&pic, canister_id, ledger_id, user, reward_amount.clone())
        .expect("Reward pool should succeed");
    
    let initial_time = pic.get_time().as_nanos_since_unix_epoch() / 1_000_000_000;
    let current_episode = initial_time / EPISODE_DURATION;
    let last_reward_episode = (initial_time + EPISODE_DURATION * 12) / EPISODE_DURATION;
    let exact_reward_duration = (last_reward_episode + 1) * EPISODE_DURATION - initial_time;
    
    let rewards_before = pic
        .query_call(canister_id, user, "get_deposits_rewards", encode_args((vec![0u64],)).unwrap())
        .map(|r| decode_one::<Nat>(&r).unwrap())
        .expect("Failed to get rewards before transition");
    
    let next_episode_start = (current_episode + 1) * EPISODE_DURATION;
    let time_to_next_episode = next_episode_start - initial_time;
    advance_time(&pic, time_to_next_episode + 1);
    

    // Get rewards after episode transition
    let rewards_after_transition = pic
        .query_call(canister_id, user, "get_deposits_rewards", encode_args((vec![0u64],)).unwrap())
        .map(|r| decode_one::<Nat>(&r).unwrap())
        .expect("Failed to get rewards after transition");
    
    // Calculate expected reward increase during episode transition
    let time_elapsed = time_to_next_episode + 1;
    let reward_rate = (reward_amount.clone() * icp_canister_backend::PRECISION_SCALE.clone()) / Nat::from(exact_reward_duration);
    let expected_increase = (reward_rate.clone() * Nat::from(time_elapsed)) / icp_canister_backend::PRECISION_SCALE.clone();
    let actual_increase = rewards_after_transition.clone() - rewards_before.clone();
    assert_with_error!(&actual_increase, &expected_increase, &ALLOWED_ERROR, "Reward increase during episode transition");
    
    // Advance through several more episodes
    for _ in 0..3 {
        advance_time(&pic, EPISODE_DURATION);
    }
    
    // Get final rewards after multiple episode transitions
    let final_rewards = pic
        .query_call(canister_id, user, "get_deposits_rewards", encode_args((vec![0u64],)).unwrap())
        .map(|r| decode_one::<Nat>(&r).unwrap())
        .expect("Failed to get final rewards");
    
    // Calculate expected additional rewards from 3 more episodes
    let additional_time = EPISODE_DURATION * 3;
    let expected_additional = (reward_rate.clone() * Nat::from(additional_time)) / icp_canister_backend::PRECISION_SCALE.clone();
    let expected_final = rewards_after_transition.clone() + expected_additional;
    assert_with_error!(&final_rewards, &expected_final, &ALLOWED_ERROR, "Final rewards after multiple episodes");
    
    // Calculate expected portion of total reward based on time elapsed
    let total_time_elapsed = time_to_next_episode + 1 + (EPISODE_DURATION * 3);
    let expected_portion = (reward_amount.clone() * Nat::from(total_time_elapsed)) / Nat::from(exact_reward_duration);
    assert_with_error!(&final_rewards, &expected_portion, &ALLOWED_ERROR, "Reward portion calculation");
}

#[test]
fn test_mathematical_precision_large_numbers() {
    let (pic, canister_id, ledger_id) = setup();
    let user = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap();
    
    let large_deposit = Nat::from(1_000_000_000u64); 
    let large_reward = Nat::from(500_000_000u64);    

    let stakable_episode = get_stakable_episode(&pic, canister_id, 7);
    
    create_deposit(&pic, canister_id, ledger_id, user, large_deposit.clone(), stakable_episode);
    
    reward_pool(&pic, canister_id, ledger_id, user, large_reward.clone())
        .expect("Large reward pool should succeed");
    
    let reward_time = pic.get_time().as_nanos_since_unix_epoch() / 1_000_000_000;
    let last_reward_episode = (reward_time + EPISODE_DURATION * 12) / EPISODE_DURATION;
    let exact_reward_duration = (last_reward_episode + 1) * EPISODE_DURATION - reward_time;
    
    let intervals = vec![
        exact_reward_duration / 10, 
        exact_reward_duration / 4,     
        exact_reward_duration / 2,   
        (exact_reward_duration * 3) / 4, 
    ];
    
    let mut previous_rewards = Nat::from(0u64);
    
    for (i, interval) in intervals.iter().enumerate() {
        advance_time(&pic, if i == 0 { *interval } else { interval - intervals[i-1] });
        
        let current_rewards = pic
            .query_call(canister_id, user, "get_deposits_rewards", encode_args((vec![0u64],)).unwrap())
            .map(|r| decode_one::<Nat>(&r).unwrap())
            .expect("Failed to get rewards for large numbers");
        
        let time_diff = if i == 0 { *interval } else { interval - intervals[i-1] };
        let reward_rate = (large_reward.clone() * icp_canister_backend::PRECISION_SCALE.clone()) / Nat::from(exact_reward_duration);
        let expected_increase = (reward_rate * Nat::from(time_diff)) / icp_canister_backend::PRECISION_SCALE.clone();
        let actual_increase = current_rewards.clone() - previous_rewards.clone();
        assert_with_error!(&actual_increase, &expected_increase, &ALLOWED_ERROR, 
                          &format!("Reward increase at interval {}", i));
        
        let time_proportion = (interval * 1000u64) / exact_reward_duration; 
        let expected_reward = (large_reward.clone() * time_proportion) / Nat::from(1000u64);
        
     
        let tolerance = large_reward.clone() / Nat::from(1000u64); 
        let diff = if current_rewards > expected_reward {
            current_rewards.clone() - expected_reward.clone()
        } else {
            expected_reward.clone() - current_rewards.clone()
        };
        
        assert!(diff <= tolerance,
               "Precision error too large at interval {}: expected={}, actual={}, diff={}, tolerance={}",
               i, expected_reward, current_rewards, diff, tolerance);
        
        previous_rewards = current_rewards;
    }
    
    advance_time(&pic, exact_reward_duration - intervals[intervals.len()-1]);
    
    let final_rewards = pic
        .query_call(canister_id, user, "get_deposits_rewards", encode_args((vec![0u64],)).unwrap())
        .map(|r| decode_one::<Nat>(&r).unwrap())
        .expect("Failed to get final large rewards");
    
    assert_with_error!(&final_rewards, &large_reward, &ALLOWED_ERROR, 
                      "Large number final reward precision");
    
    let withdrawal_result = pic
        .update_call(canister_id, user, "withdraw_rewards", encode_args((vec![0u64],)).unwrap())
        .expect("Large reward withdrawal should succeed");
    
    let withdrawal: Result<Nat, icp_canister_backend::PoolError> = decode_one(&withdrawal_result).unwrap();
    let withdrawn_amount = withdrawal.expect("Large reward withdrawal should succeed");
    // Withdrawn amount should match calculated rewards exactly
    assert_with_error!(&withdrawn_amount, &final_rewards, &ALLOWED_ERROR,
                      "Withdrawn amount should match calculated rewards");
    
    let expected_withdrawn = large_reward.clone(); 
    assert_with_error!(&withdrawn_amount, &expected_withdrawn, &ALLOWED_ERROR, "Large withdrawal amount verification");
 }

 