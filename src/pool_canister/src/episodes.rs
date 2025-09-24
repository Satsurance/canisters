use crate::storage::*;
use crate::types::{Episode, StorableNat};
use crate::{EPISODE_DURATION, MAX_ACTIVE_EPISODES};
use candid::Nat;
use ic_cdk_timers;

pub fn get_current_episode() -> u64 {
    ic_cdk::api::time() / 1_000_000_000 / EPISODE_DURATION
}

fn get_last_processed_episode() -> u64 {
    let last_updated = LAST_TIME_UPDATED.with(|cell| cell.borrow().get().clone());
    last_updated / EPISODE_DURATION
}

pub fn is_episode_active(episode_id: u64) -> bool {
    let current_episode = get_current_episode();
    episode_id >= current_episode && episode_id < current_episode + MAX_ACTIVE_EPISODES
}

pub fn is_episode_stakable(episode_id: u64) -> bool {
    episode_id % 3 == 2
}

#[ic_cdk::query]
pub fn get_pool_reward_rate() -> Nat {
    POOL_REWARD_RATE.with(|cell| cell.borrow().get().clone().0)
}

#[ic_cdk::query]
pub fn get_pool_state() -> crate::types::PoolState {
    POOL_STATE.with(|state| state.borrow().get().clone())
}

#[ic_cdk::query]
pub fn get_current_episode_id() -> u64 {
    get_current_episode()
}

#[ic_cdk::query]
pub fn get_episode(episode_id: u64) -> Option<Episode> {
    EPISODES.with(|episodes| episodes.borrow().get(&episode_id))
}

#[ic_cdk::update]
pub fn update_episodes_state() {
    process_episodes();
}

pub fn process_episodes() {
    let current_episode = get_current_episode();
    let last_processed_episode = get_last_processed_episode();
    let current_time = ic_cdk::api::time() / 1_000_000_000;
    let last_updated_time = LAST_TIME_UPDATED.with(|cell| cell.borrow().get().clone());

    if current_time == last_updated_time {
        return;
    }

    let mut updated_rewards_at = last_updated_time;

    EPISODES.with(|episodes| {
        let mut episodes_ref = episodes.borrow_mut();
        POOL_STATE.with(|state| {
            let mut pool_state = state.borrow().get().clone();
            for episode_id in last_processed_episode..current_episode {
                if let Some(mut episode) = episodes_ref.get(&episode_id) {
                    let episode_finish_time = (episode_id + 1) * EPISODE_DURATION;
                    let reward_rate_contribution =
                        reward_rate_per_share(updated_rewards_at, episode_finish_time);
                    ACCUMULATED_REWARD_PER_SHARE.with(|cell| {
                        let current_acc = cell.borrow().get().clone().0;
                        cell.borrow_mut()
                            .set(StorableNat(current_acc + reward_rate_contribution.clone()))
                            .ok();
                    });
                    updated_rewards_at = episode_finish_time;
                    POOL_REWARD_RATE.with(|cell| {
                        let current_rate = cell.borrow().get().clone().0;
                        cell.borrow_mut()
                            .set(StorableNat(current_rate - episode.reward_decrease.clone()))
                            .ok();
                    });
                    episode.acc_reward_per_share_on_expire =
                        ACCUMULATED_REWARD_PER_SHARE.with(|cell| cell.borrow().get().clone().0);
                    episodes_ref.insert(episode_id, episode.clone());
                    pool_state.total_assets -= episode.assets_staked.clone();
                    pool_state.total_shares -= episode.episode_shares.clone();
                    state.borrow_mut().set(pool_state.clone()).ok();
                }
            }
        });
    });

    let final_reward_contribution = reward_rate_per_share(updated_rewards_at, current_time);
    ACCUMULATED_REWARD_PER_SHARE.with(|cell| {
        let current_acc = cell.borrow().get().clone().0;
        cell.borrow_mut()
            .set(StorableNat(current_acc + final_reward_contribution))
            .ok();
    });

    LAST_TIME_UPDATED.with(|cell| {
        cell.borrow_mut().set(current_time).ok();
    });
}

fn reward_rate_per_share(updated_rewards_at: u64, finish_time: u64) -> Nat {
    let pool_state = POOL_STATE.with(|state| state.borrow().get().clone());
    let pool_reward_rate = POOL_REWARD_RATE.with(|cell| cell.borrow().get().clone().0);

    if pool_state.total_assets == Nat::from(0u64) || pool_state.total_shares == Nat::from(0u64) {
        return Nat::from(0u64);
    }

    let time_diff = Nat::from(finish_time - updated_rewards_at);
    let result = (pool_reward_rate.clone() * time_diff.clone()) / pool_state.total_shares.clone();
    result
}

pub fn setup_episode_timer() {
    let current_time = ic_cdk::api::time() / 1_000_000_000;
    let current_episode = current_time / EPISODE_DURATION;
    let next_episode_start = (current_episode + 1) * EPISODE_DURATION;
    let time_to_next_episode = next_episode_start - current_time;

    ic_cdk_timers::set_timer(std::time::Duration::from_secs(time_to_next_episode), || {
        process_episodes();
        setup_episode_timer();
    });
}
