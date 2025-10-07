use crate::episodes::process_episodes;
use crate::ledger::{get_reward_subaccount, get_subaccount_balance, transfer_icrc1};
use crate::storage::*;
use crate::types::{Episode, PoolError, StorableNat};
use crate::{EPISODE_DURATION, PRECISION_SCALE};
use candid::Nat;

pub fn collect_deposit_rewards(deposit_ids: Vec<u64>, update_deposit: bool) -> Nat {
    let current_accumulated_reward =
        ACCUMULATED_REWARD_PER_SHARE.with(|cell| cell.borrow().get().clone().0);

    let mut total_rewards = Nat::from(0u64);
    DEPOSITS.with(|deposits| {
        let mut deposits_ref = deposits.borrow_mut();

        for &deposit_id in &deposit_ids {
            if let Some(mut deposit) = deposits_ref.get(&deposit_id) {
                let reward_per_share_to_use = current_accumulated_reward.clone();
                let reward_diff = reward_per_share_to_use - deposit.reward_per_share.clone();
                let total_earned =
                    (deposit.shares.clone() * reward_diff.clone()) / PRECISION_SCALE.clone();
                let uncollected = total_earned.clone() - deposit.rewards_collected.clone();
                total_rewards += uncollected.clone();

                if update_deposit {
                    deposit.rewards_collected += uncollected;
                    deposits_ref.insert(deposit_id, deposit);
                }
            }
        }
    });

    total_rewards
}

#[ic_cdk::query]
pub fn get_deposits_rewards(deposit_ids: Vec<u64>) -> Nat {
    collect_deposit_rewards(deposit_ids, false)
}

#[ic_cdk::update]
pub async fn withdraw_rewards(deposit_ids: Vec<u64>) -> Result<Nat, PoolError> {
    let caller = ic_cdk::api::caller();

    let user_deposit_ids = USER_DEPOSITS.with(|user_deposits| {
        user_deposits
            .borrow()
            .get(&caller)
            .map(|deposits| deposits.0.clone())
            .unwrap_or_default()
    });

    for &deposit_id in &deposit_ids {
        if !user_deposit_ids.contains(&deposit_id) {
            return Err(PoolError::NotOwner);
        }
    }

    let total_withdrawable = collect_deposit_rewards(deposit_ids, true);

    transfer_icrc1(None, caller, total_withdrawable.clone()).await?;

    Ok(total_withdrawable)
}

#[ic_cdk::update]
pub async fn reward_pool() -> Result<(), PoolError> {
    process_episodes();

    let reward_subaccount = get_reward_subaccount();
    let balance = get_subaccount_balance(reward_subaccount.to_vec()).await?;

    transfer_icrc1(
        Some(reward_subaccount.to_vec()),
        ic_cdk::api::id(),
        balance.clone(),
    )
    .await?;

    let amount = balance - crate::TRANSFER_FEE.clone();

    let current_time = ic_cdk::api::time() / 1_000_000_000;
    let last_reward_episode = (current_time + EPISODE_DURATION * 12) / EPISODE_DURATION;
    let reward_duration = (last_reward_episode + 1) * EPISODE_DURATION - current_time;

    let reward_rate_increase = (amount * PRECISION_SCALE.clone()) / Nat::from(reward_duration);

    POOL_REWARD_RATE.with(|cell| {
        let current_rate = cell.borrow().get().clone().0;
        cell.borrow_mut()
            .set(StorableNat(current_rate + reward_rate_increase.clone()))
            .ok();
    });

    EPISODES.with(|episodes| {
        let mut episodes_ref = episodes.borrow_mut();
        let target_episode_id = last_reward_episode;
        let mut episode = episodes_ref.get(&target_episode_id).unwrap_or(Episode {
            episode_shares: Nat::from(0u64),
            assets_staked: Nat::from(0u64),
            reward_decrease: Nat::from(0u64),
            coverage_decrease: Nat::from(0u64),
            acc_reward_per_share_on_expire: Nat::from(0u64),
        });
        episode.reward_decrease += reward_rate_increase;
        episodes_ref.insert(target_episode_id, episode);
    });

    Ok(())
}

pub fn reward_pool_with_duration(amount: Nat, coverage_duration: u64) {
    let current_time = ic_cdk::api::time() / 1_000_000_000;
    let last_reward_episode = (current_time + coverage_duration) / EPISODE_DURATION;
    let reward_duration = (last_reward_episode + 1) * EPISODE_DURATION - current_time;

    let reward_rate_increase = (amount * PRECISION_SCALE.clone()) / Nat::from(reward_duration);

    POOL_REWARD_RATE.with(|cell| {
        let current_rate = cell.borrow().get().clone().0;
        cell.borrow_mut()
            .set(StorableNat(current_rate + reward_rate_increase.clone()))
            .ok();
    });

    EPISODES.with(|episodes| {
        let mut episodes_ref = episodes.borrow_mut();
        let target_episode_id = last_reward_episode + 1;
        let mut episode = episodes_ref.get(&target_episode_id).unwrap_or(Episode {
            episode_shares: Nat::from(0u64),
            assets_staked: Nat::from(0u64),
            reward_decrease: Nat::from(0u64),
            coverage_decrease: Nat::from(0u64),
            acc_reward_per_share_on_expire: Nat::from(0u64),
        });
        episode.reward_decrease += reward_rate_increase;
        episodes_ref.insert(target_episode_id, episode);
    });
}