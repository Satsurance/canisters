use crate::episodes::{
    get_current_episode, is_episode_active, is_episode_stakable, process_episodes,
};
use crate::ledger::{get_deposit_subaccount, get_subaccount_balance, transfer_icrc1};
use crate::rewards::collect_deposit_rewards;
use crate::storage::*;
use crate::types::{Deposit, Episode, PoolError, UserDepositInfo, UserDeposits};
use crate::MINIMUM_DEPOSIT_AMOUNT;
use candid::{Nat, Principal};

#[ic_cdk::query]
pub fn get_user_deposits(user: Principal) -> Vec<UserDepositInfo> {
    let deposit_ids = USER_DEPOSITS.with(|user_deposits| match user_deposits.borrow().get(&user) {
        Some(deposits) => deposits.0.clone(),
        None => return Vec::new(),
    });

    EPISODES.with(|episodes| {
        let episodes_ref = episodes.borrow();
        DEPOSITS.with(|deposits| {
            let deposits_ref = deposits.borrow();
            deposit_ids
                .iter()
                .filter_map(|&deposit_id| {
                    deposits_ref.get(&deposit_id).map(|deposit| {
                        let episode_data = episodes_ref
                            .get(&deposit.episode)
                            .expect("Episode should exist for deposit");

                        let amount = deposit.shares.clone() * episode_data.assets_staked.clone()
                            / episode_data.episode_shares.clone();

                        UserDepositInfo {
                            deposit_id,
                            episode: deposit.episode,
                            shares: deposit.shares.clone(),
                            amount,
                        }
                    })
                })
                .collect()
        })
    })
}

#[ic_cdk::query]
pub fn get_deposit(id: u64) -> Option<Deposit> {
    DEPOSITS.with(|map| map.borrow().get(&id).map(|deposit| deposit.clone()))
}

#[ic_cdk::update]
pub async fn deposit(user: Principal, episode_id: u64) -> Result<(), PoolError> {
    process_episodes();

    if !is_episode_active(episode_id) {
        return Err(PoolError::EpisodeNotActive);
    }

    if !is_episode_stakable(episode_id) {
        return Err(PoolError::EpisodeNotStakable);
    }

    let subaccount = get_deposit_subaccount(user, episode_id);
    let balance = get_subaccount_balance(subaccount.to_vec()).await?;

    if balance <= MINIMUM_DEPOSIT_AMOUNT.clone() {
        return Err(PoolError::InsufficientBalance);
    }

    transfer_icrc1(
        Some(subaccount.to_vec()),
        ic_cdk::api::id(),
        balance.clone(),
    )
    .await?;

    let transfer_amount = balance - crate::TRANSFER_FEE.clone();

    let current_pool_state = POOL_STATE.with(|state| state.borrow().get().clone());
    let new_shares = if current_pool_state.total_shares == Nat::from(0u64) {
        transfer_amount.clone()
    } else {
        transfer_amount.clone() * current_pool_state.total_shares.clone()
            / current_pool_state.total_assets.clone()
    };

    let deposit_id = DEPOSIT_COUNTER.with(|counter| {
        let current = counter.borrow().get().clone();
        let new_counter = current + 1;
        counter.borrow_mut().set(new_counter).ok();
        current
    });

    let current_accumulated_reward =
        ACCUMULATED_REWARD_PER_SHARE.with(|cell| cell.borrow().get().clone().0);

    let deposit = Deposit {
        episode: episode_id,
        shares: new_shares.clone(),
        reward_per_share: current_accumulated_reward,
        rewards_collected: Nat::from(0u64),
    };

    add_deposit(deposit_id, deposit, user, transfer_amount.clone(), true);

    Ok(())
}

#[ic_cdk::update]
pub async fn withdraw(deposit_id: u64) -> Result<(), PoolError> {
    process_episodes();
    let caller = ic_cdk::api::caller();
    let current_episode = get_current_episode();

    let deposit = DEPOSITS.with(|deposits| deposits.borrow().get(&deposit_id).clone());
    let deposit = match deposit {
        Some(d) => d,
        None => return Err(PoolError::NoDeposit),
    };

    let is_owner = USER_DEPOSITS.with(|user_deposits| {
        user_deposits
            .borrow()
            .get(&caller)
            .map(|deposits| deposits.0.contains(&deposit_id))
            .unwrap_or(false)
    });

    if !is_owner {
        return Err(PoolError::NotOwner);
    }

    if deposit.episode >= current_episode {
        return Err(PoolError::TimelockNotExpired);
    }

    let pending_rewards = collect_deposit_rewards(vec![deposit_id], true);

    let episode_data = EPISODES.with(|episodes| {
        episodes
            .borrow()
            .get(&deposit.episode)
            .ok_or(PoolError::NoDeposit)
    })?;

    let withdrawal_amount = deposit.shares.clone() * episode_data.assets_staked.clone()
        / episode_data.episode_shares.clone();
    let total_transfer_amount = withdrawal_amount.clone() + pending_rewards.clone();

    let transfer_result = transfer_icrc1(None, caller, total_transfer_amount).await;
    if transfer_result.is_err() {
        return Err(PoolError::TransferFailed);
    }

    DEPOSITS.with(|deposits| deposits.borrow_mut().remove(&deposit_id));

    USER_DEPOSITS.with(|user_deposits| {
        let mut user_deposits = user_deposits.borrow_mut();
        if let Some(mut user_deposits_list) = user_deposits.get(&caller) {
            user_deposits_list.0.retain(|&id| id != deposit_id);
            user_deposits.insert(caller, user_deposits_list);
        }
    });

    EPISODES.with(|episodes| {
        let mut episodes_ref = episodes.borrow_mut();
        if let Some(mut episode) = episodes_ref.get(&deposit.episode) {
            episode.episode_shares -= deposit.shares.clone();
            episode.assets_staked -= withdrawal_amount.clone();

            if episode.episode_shares == Nat::from(0u64) {
                episodes_ref.remove(&deposit.episode);
            } else {
                episodes_ref.insert(deposit.episode, episode);
            }
        }
    });

    Ok(())
}

pub fn add_deposit(
    deposit_id: u64,
    deposit: Deposit,
    user: Principal,
    assets_amount: Nat,
    update_pool_stats: bool,
) {
    DEPOSITS.with(|deposits| {
        deposits.borrow_mut().insert(deposit_id, deposit.clone());
    });

    USER_DEPOSITS.with(|user_deposits| {
        let mut user_deposits = user_deposits.borrow_mut();
        let mut user_deposits_list = user_deposits.get(&user).unwrap_or(UserDeposits(vec![]));
        user_deposits_list.0.push(deposit_id);
        user_deposits.insert(user, user_deposits_list);
    });

    EPISODES.with(|episodes| {
        let mut episodes_ref = episodes.borrow_mut();
        let mut episode = episodes_ref.get(&deposit.episode).unwrap_or(Episode {
            episode_shares: Nat::from(0u64),
            assets_staked: Nat::from(0u64),
            reward_decrease: Nat::from(0u64),
            acc_reward_per_share_on_expire: Nat::from(0u64),
        });
        episode.episode_shares += deposit.shares.clone();
        episode.assets_staked += assets_amount.clone();
        episodes_ref.insert(deposit.episode, episode);
    });

    if update_pool_stats {
        POOL_STATE.with(|state| {
            let mut pool_state = state.borrow().get().clone();
            pool_state.total_assets += assets_amount.clone();
            pool_state.total_shares += deposit.shares.clone();
            state.borrow_mut().set(pool_state).ok();
        });
    }
}
