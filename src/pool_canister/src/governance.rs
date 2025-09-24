use crate::episodes::get_current_episode;
use crate::ledger::transfer_icrc1;
use crate::storage::*;
use crate::types::PoolError;
use crate::MAX_ACTIVE_EPISODES;
use candid::{Nat, Principal};

#[ic_cdk::update]
pub fn set_executor_principal(executor: Principal) -> Result<(), PoolError> {
    let caller = ic_cdk::api::caller();
    let current_executor = EXECUTOR_PRINCIPAL.with(|cell| cell.borrow().get().clone());

    if caller != current_executor {
        return Err(PoolError::NotSlashingExecutor);
    }

    EXECUTOR_PRINCIPAL.with(|cell| {
        cell.borrow_mut().set(executor).ok();
    });
    Ok(())
}

#[ic_cdk::update]
pub async fn slash(receiver: Principal, amount: Nat) -> Result<(), PoolError> {
    let caller = ic_cdk::api::caller();
    let executor_principal = EXECUTOR_PRINCIPAL.with(|cell| cell.borrow().get().clone());

    if caller != executor_principal {
        return Err(PoolError::NotSlashingExecutor);
    }

    let current_episode = get_current_episode();

    let accumulated_slashed = POOL_STATE.with(|state| {
        let mut pool_state_ref = state.borrow_mut();
        let mut pool_state = pool_state_ref.get().clone();
        let mut accumulated_slashed = Nat::from(0u64);

        EPISODES.with(|episodes| {
            let mut episodes_ref = episodes.borrow_mut();

            for i in current_episode..(current_episode + MAX_ACTIVE_EPISODES) {
                if let Some(mut episode) = episodes_ref.get(&i) {
                    let slash_amount_for_episode = amount.clone() * episode.assets_staked.clone()
                        / pool_state.total_assets.clone();
                    accumulated_slashed += slash_amount_for_episode.clone();
                    episode.assets_staked -= slash_amount_for_episode.clone();
                    episodes_ref.insert(i, episode);
                }
            }
        });

        pool_state.total_assets -= accumulated_slashed.clone();
        pool_state_ref.set(pool_state).ok();

        accumulated_slashed
    });

    transfer_icrc1(None, receiver, accumulated_slashed).await?;

    Ok(())
}
