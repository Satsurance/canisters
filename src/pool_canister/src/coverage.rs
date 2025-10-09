use crate::episodes::{get_current_episode, process_episodes};
use crate::ledger::{get_purchase_subaccount, get_subaccount_balance, transfer_icrc1};
use crate::rewards::reward_pool_with_duration;
use crate::storage::*;
use crate::types::{Coverage, Episode, PoolError, Product, StorableNat, UserCoverages};
use crate::{EPISODE_DURATION, MAX_ACTIVE_EPISODES};
use candid::{Nat, Principal};

const BASIS_POINTS: u64 = 10_000;
const SECONDS_PER_YEAR: u64 = 365 * 24 * 60 * 60;

fn update_product_allocation(product: &mut Product) {
    product.allocation = compute_current_product_allocation(product);
    product.last_allocation_update = ic_cdk::api::time() / 1_000_000_000;
}

fn compute_current_product_allocation(product: &Product) -> Nat {
    let last_updated_episode = product.last_allocation_update / EPISODE_DURATION;
    let current_episode = get_current_episode();

    if last_updated_episode == current_episode {
        return product.allocation.clone();
    }

    if current_episode >= last_updated_episode + MAX_ACTIVE_EPISODES {
        return Nat::from(0u64);
    }

    EPISODE_ALLOCATION_CUT.with(|cuts| {
        let cuts_ref = cuts.borrow();
        let mut allocation_cut = Nat::from(0u64);
        
        for i in last_updated_episode..=current_episode {
            if let Some(cut) = cuts_ref.get(&(product.product_id, i)) {
                allocation_cut += cut.0.clone();
            }
        }

        product.allocation.clone() - allocation_cut
    })
}

fn verify_product_allocation(last_covered_episode: u64, requested_allocation: Nat) -> bool {
    let current_episode = get_current_episode();
    let pool_state = POOL_STATE.with(|state| state.borrow().get().clone());

    if pool_state.total_shares == Nat::from(0u64) {
        return false;
    }

    EPISODES.with(|episodes| {
        let episodes_ref = episodes.borrow();
        let mut available_allocation = Nat::from(0u64);

        for i in last_covered_episode..(current_episode + MAX_ACTIVE_EPISODES) {
            let episode_allocation = if let Some(episode) = episodes_ref.get(&i) {
                if episode.episode_shares > Nat::from(0u64) {
                    episode.episode_shares.clone() * pool_state.total_assets.clone()
                        / pool_state.total_shares.clone()
                } else {
                    Nat::from(0u64)
                }
            } else {
                Nat::from(0u64)
            };

            available_allocation += episode_allocation;

            if available_allocation >= requested_allocation {
                return true;
            }
        }

        false
    })
}

#[ic_cdk::update]
pub async fn purchase_coverage(
    product_id: u64,
    covered_account: Principal,
    coverage_duration: u64,
    coverage_amount: Nat,
) -> Result<(), PoolError> {
    let mut product = PRODUCTS
        .with(|products| products.borrow().get(&product_id))
        .ok_or(PoolError::ProductNotFound)?;

    if !product.active {
        return Err(PoolError::ProductNotActive);
    }

    if coverage_duration > product.max_coverage_duration {
        return Err(PoolError::CoverageDurationTooLong);
    }

    if coverage_duration < EPISODE_DURATION {
        return Err(PoolError::CoverageDurationTooShort);
    }

    if covered_account == Principal::anonymous() {
        return Err(PoolError::InvalidProductParameters);
    }

    process_episodes();
    update_product_allocation(&mut product);

    let current_time = ic_cdk::api::time() / 1_000_000_000;
    let last_covered_episode = (current_time + coverage_duration) / EPISODE_DURATION;

    let new_total_allocation = coverage_amount.clone() + product.allocation.clone();
    let required_pool_allocation = new_total_allocation.clone() * Nat::from(BASIS_POINTS)
        / Nat::from(product.max_pool_allocation_percent);

    if !verify_product_allocation(last_covered_episode, required_pool_allocation) {
        return Err(PoolError::NotEnoughAssetsToCover);
    }

    EPISODE_ALLOCATION_CUT.with(|cuts| {
        let mut cuts_ref = cuts.borrow_mut();
        let key = (product_id, last_covered_episode);
        let current_cut = cuts_ref
            .get(&key)
            .unwrap_or(StorableNat(Nat::from(0u64)));
        cuts_ref.insert(key, StorableNat(current_cut.0 + coverage_amount.clone()));
    });

    product.allocation += coverage_amount.clone();
    PRODUCTS.with(|products| {
        products.borrow_mut().insert(product_id, product.clone());
    });

    TOTAL_COVER_ALLOCATION.with(|cell| {
        let current_allocation = cell.borrow().get().clone().0;
        cell.borrow_mut()
            .set(StorableNat(current_allocation + coverage_amount.clone()))
            .ok();
    });

    EPISODES.with(|episodes| {
        let mut episodes_ref = episodes.borrow_mut();
        let mut episode = episodes_ref.get(&last_covered_episode).unwrap_or(Episode {
            episode_shares: Nat::from(0u64),
            assets_staked: Nat::from(0u64),
            reward_decrease: Nat::from(0u64),
            coverage_decrease: Nat::from(0u64),
            acc_reward_per_share_on_expire: Nat::from(0u64),
        });
        episode.coverage_decrease += coverage_amount.clone();
        episodes_ref.insert(last_covered_episode, episode);
    });

    let premium_amount = (Nat::from(coverage_duration) * Nat::from(product.annual_percent)
        * coverage_amount.clone())
        / (Nat::from(SECONDS_PER_YEAR) * Nat::from(BASIS_POINTS));

    let caller = ic_cdk::caller();
    let purchase_subaccount = get_purchase_subaccount(caller, product_id);
    let subaccount_balance = get_subaccount_balance(purchase_subaccount.to_vec()).await?;

    if subaccount_balance < premium_amount {
        return Err(PoolError::InsufficientBalance);
    }

    // TODO: add refund back to user account on error or manage it different way
    transfer_icrc1(
        Some(purchase_subaccount.to_vec()),
        ic_cdk::api::id(),
        premium_amount.clone(),
    )
    .await?;

    let reward_amount = premium_amount.clone() - crate::TRANSFER_FEE.clone();
    reward_pool_with_duration(reward_amount, coverage_duration);

    let coverage_id = COVERAGE_COUNTER.with(|counter| {
        let current = counter.borrow().get().clone();
        let new_counter = current + 1;
        counter.borrow_mut().set(new_counter).ok();
        current
    });

    let coverage = Coverage {
        coverage_id,
        buyer: caller,
        covered_account,
        product_id,
        coverage_amount: coverage_amount.clone(),
        premium_amount: premium_amount,
        start_time: current_time,
        end_time: current_time + coverage_duration,
    };

    COVERAGES.with(|coverages| {
        coverages.borrow_mut().insert(coverage_id, coverage);
    });

    USER_COVERAGES.with(|user_coverages| {
        let mut user_coverages_ref = user_coverages.borrow_mut();
        let mut user_coverage_list = user_coverages_ref
            .get(&caller)
            .unwrap_or(UserCoverages(vec![]));
        user_coverage_list.0.push(coverage_id);
        user_coverages_ref.insert(caller, user_coverage_list);
    });

    Ok(())
}

#[ic_cdk::update]
pub fn create_product(
    name: String,
    annual_percent: u64,
    max_coverage_duration: u64,
    max_pool_allocation_percent: u64,
) -> Result<u64, PoolError> {
    let caller = ic_cdk::caller();
    let pool_manager = POOL_MANAGER_PRINCIPAL.with(|cell| cell.borrow().get().clone());
    
    if caller != pool_manager {
        return Err(PoolError::NotPoolManager);
    }

    if max_coverage_duration < EPISODE_DURATION {
        return Err(PoolError::InvalidProductParameters);
    }

    if max_coverage_duration >= (MAX_ACTIVE_EPISODES - 1) * EPISODE_DURATION {
        return Err(PoolError::InvalidProductParameters);
    }

    if max_pool_allocation_percent > BASIS_POINTS {
        return Err(PoolError::InvalidProductParameters);
    }

    if annual_percent == 0 {
        return Err(PoolError::InvalidProductParameters);
    }

    let product_id = PRODUCT_COUNTER.with(|counter| {
        let current = counter.borrow().get().clone();
        let new_counter = current + 1;
        counter.borrow_mut().set(new_counter).ok();
        current
    });

    let current_time = ic_cdk::api::time() / 1_000_000_000;

    let product = Product {
        name,
        product_id,
        annual_percent,
        max_coverage_duration,
        max_pool_allocation_percent,
        allocation: Nat::from(0u64),
        last_allocation_update: current_time,
        active: true,
    };

    PRODUCTS.with(|products| {
        products.borrow_mut().insert(product_id, product);
    });

    Ok(product_id)
}

#[ic_cdk::update]
pub fn set_product(
    product_id: u64,
    annual_percent: u64,
    max_coverage_duration: u64,
    max_pool_allocation_percent: u64,
    active: bool,
) -> Result<(), PoolError> {
    let caller = ic_cdk::caller();
    let pool_manager = POOL_MANAGER_PRINCIPAL.with(|cell| cell.borrow().get().clone());
    
    if caller != pool_manager {
        return Err(PoolError::NotPoolManager);
    }

    if max_coverage_duration < EPISODE_DURATION {
        return Err(PoolError::InvalidProductParameters);
    }

    if max_coverage_duration >= (MAX_ACTIVE_EPISODES - 1) * EPISODE_DURATION {
        return Err(PoolError::InvalidProductParameters);
    }

    if max_pool_allocation_percent > BASIS_POINTS {
        return Err(PoolError::InvalidProductParameters);
    }

    if annual_percent == 0 {
        return Err(PoolError::InvalidProductParameters);
    }

    let mut product = PRODUCTS
        .with(|products| products.borrow().get(&product_id))
        .ok_or(PoolError::ProductNotFound)?;

    product.annual_percent = annual_percent;
    product.max_coverage_duration = max_coverage_duration;
    product.max_pool_allocation_percent = max_pool_allocation_percent;
    product.active = active;

    PRODUCTS.with(|products| {
        products.borrow_mut().insert(product_id, product);
    });

    Ok(())
}


#[ic_cdk::query]
pub fn get_products() -> Vec<Product> {
    PRODUCTS.with(|products| {
        products.borrow()
            .iter()
            .map(|(_, product)| {
                let mut updated_product = product.clone();
                updated_product.allocation = compute_current_product_allocation(&product);
                updated_product
            })
            .collect()
    })
}

#[ic_cdk::query]
pub fn get_total_cover_allocation() -> Nat {
    TOTAL_COVER_ALLOCATION.with(|cell| cell.borrow().get().clone().0)
}


#[ic_cdk::query]
pub fn get_coverages(user: Principal) -> Vec<Coverage> {
    let coverage_ids = USER_COVERAGES.with(|user_coverages| {
        match user_coverages.borrow().get(&user) {
            Some(coverages) => coverages.0.clone(),
            None => return Vec::new(),
        }
    });

    COVERAGES.with(|coverages| {
        let coverages_ref = coverages.borrow();
        coverage_ids
            .iter()
            .filter_map(|&coverage_id| coverages_ref.get(&coverage_id))
            .collect()
    })
}

#[ic_cdk::query]
pub fn get_coverage(coverage_id: u64) -> Option<Coverage> {
    COVERAGES.with(|coverages| coverages.borrow().get(&coverage_id))
}
