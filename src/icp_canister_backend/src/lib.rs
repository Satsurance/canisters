use candid::{Nat, Principal};
use ic_cdk::api::call::call;
use ic_cdk_timers;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, StableCell};
use lazy_static::lazy_static;
use sha2::{Digest, Sha256};
use std::cell::RefCell;

pub const EPISODE_DURATION: u64 = 91 * 24 * 60 * 60 / 3;
const MAX_ACTIVE_EPISODES: u64 = 24;

lazy_static! {
    pub static ref TRANSFER_FEE: Nat = Nat::from(10_000u64);
    pub static ref MINIMUM_DEPOSIT_AMOUNT: Nat = Nat::from(100_000u64);
}

pub mod types;

pub use types::{Account, Deposit, Episode, PoolError, PoolState, TransferArg, TransferError};

type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static TOKEN_ID: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
            Principal::anonymous()
        ).expect("Failed to initialize StableCell")
    );

    static DEPOSIT_COUNTER: RefCell<StableCell<u64, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))),
            0u64
        ).expect("Failed to initialize StableCell")
    );

    static DEPOSITS: RefCell<StableBTreeMap<u64, Deposit, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2))),
        )
    );

    static USER_DEPOSITS: RefCell<StableBTreeMap<Principal, types::UserDeposits, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3))),
        )
    );

    static POOL_STATE: RefCell<StableCell<PoolState, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4))),
            PoolState {
                total_assets: Nat::from(0u64),
                total_shares: Nat::from(0u64),
            }
        ).expect("Failed to initialize PoolState")
    );

    static EPISODES: RefCell<StableBTreeMap<u64, Episode, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(5))),
        )
    );

    static LAST_TIME_UPDATED: RefCell<StableCell<u64, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(6))),
            0u64
        ).expect("Failed to initialize LAST_TIME_UPDATED")
    );
}

fn get_current_episode() -> u64 {
    ic_cdk::api::time() / 1_000_000_000 / EPISODE_DURATION
}

fn get_last_processed_episode() -> u64 {
    let last_updated = LAST_TIME_UPDATED.with(|cell| cell.borrow().get().clone());
    last_updated / EPISODE_DURATION
}

fn is_episode_active(episode_id: u64) -> bool {
    let current_episode = get_current_episode();
    episode_id >= current_episode && episode_id < current_episode + MAX_ACTIVE_EPISODES
}

#[ic_cdk::query]
pub fn get_deposit_subaccount(user: Principal, episode: u64) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(user.as_slice());
    hasher.update(episode.to_be_bytes());
    hasher.finalize().into()
}

#[ic_cdk::query]
pub fn get_current_episode_id() -> u64 {
    get_current_episode()
}

#[ic_cdk::query]
pub fn get_episode(episode_id: u64) -> Option<Episode> {
    EPISODES.with(|episodes| episodes.borrow().get(&episode_id))
}

#[ic_cdk::query]
pub fn get_user_deposits(user: Principal) -> Vec<types::UserDepositInfo> {
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

                        types::UserDepositInfo {
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
pub fn get_deposit(id: u64) -> Option<types::Deposit> {
    DEPOSITS.with(|map| map.borrow().get(&id).map(|deposit| deposit.clone()))
}

#[ic_cdk::query]
pub fn get_pool_state() -> PoolState {
    POOL_STATE.with(|state| state.borrow().get().clone())
}

fn add_deposit(deposit_id: u64, deposit: types::Deposit, user: Principal, assets_amount: Nat, update_pool_stats: bool) {
    DEPOSITS.with(|deposits| {
        deposits.borrow_mut().insert(deposit_id, deposit.clone());
    });

    USER_DEPOSITS.with(|user_deposits| {
        let mut user_deposits = user_deposits.borrow_mut();
        let mut user_deposits_list = user_deposits
            .get(&user)
            .unwrap_or(types::UserDeposits(vec![]));
        user_deposits_list.0.push(deposit_id);
        user_deposits.insert(user, user_deposits_list);
    });

    if update_pool_stats {
        EPISODES.with(|episodes| {
            let mut episodes_ref = episodes.borrow_mut();
            let mut episode = episodes_ref.get(&deposit.episode).unwrap_or(Episode {
                episode_shares: Nat::from(0u64),
                assets_staked: Nat::from(0u64),
            });
            episode.episode_shares += deposit.shares.clone();
            episode.assets_staked += assets_amount.clone();
            episodes_ref.insert(deposit.episode, episode);
        });

        POOL_STATE.with(|state| {
            let mut pool_state = state.borrow().get().clone();
            pool_state.total_assets += assets_amount.clone();
            pool_state.total_shares += deposit.shares.clone();
            state.borrow_mut().set(pool_state).ok();
        });
    }
}

#[ic_cdk::init]
pub fn init(token_id: Principal) {
    TOKEN_ID.with(|cell| {
        cell.borrow_mut().set(token_id).ok();
    });

    let current_time = ic_cdk::api::time() / 1_000_000_000;
    LAST_TIME_UPDATED.with(|cell| {
        cell.borrow_mut().set(current_time).ok();
    });

    setup_episode_timer();
}

fn process_episodes() {
    let current_episode = get_current_episode();
    let last_processed_episode = get_last_processed_episode();

    let mut total_assets_to_subtract = Nat::from(0u64);
    let mut total_shares_to_subtract = Nat::from(0u64);

    EPISODES.with(|episodes| {
        let episodes_ref = episodes.borrow();

        for episode_id in (last_processed_episode)..current_episode {
            if let Some(episode) = episodes_ref.get(&episode_id) {
                total_assets_to_subtract += episode.assets_staked.clone();
                total_shares_to_subtract += episode.episode_shares.clone();
            }
        }
    });

    if total_assets_to_subtract > Nat::from(0u64) || total_shares_to_subtract > Nat::from(0u64) {
        POOL_STATE.with(|state| {
            let mut pool_state = state.borrow().get().clone();
            pool_state.total_assets -= total_assets_to_subtract;
            pool_state.total_shares -= total_shares_to_subtract;
            state.borrow_mut().set(pool_state).ok();
        });
    }

    let current_time = ic_cdk::api::time() / 1_000_000_000;
    LAST_TIME_UPDATED.with(|cell| {
        cell.borrow_mut().set(current_time).ok();
    });
}

fn setup_episode_timer() {
    let current_time = ic_cdk::api::time() / 1_000_000_000;
    let current_episode = current_time / EPISODE_DURATION;
    let next_episode_start = (current_episode + 1) * EPISODE_DURATION;
    let time_to_next_episode = next_episode_start - current_time;

    ic_cdk_timers::set_timer(std::time::Duration::from_secs(time_to_next_episode), || {
        process_episodes();
        setup_episode_timer();
    });
}

#[ic_cdk::update]
pub fn update_episodes_state() {
    process_episodes();
}

#[ic_cdk::update]
pub async fn deposit(user: Principal, episode_id: u64) -> Result<(), types::PoolError> {
    if !is_episode_active(episode_id) {
        return Err(types::PoolError::EpisodeNotActive);
    }

    let ledger_principal = TOKEN_ID
        .with(|cell| {
            let stored = cell.borrow().get().clone();
            Some(stored)
        })
        .ok_or(types::PoolError::LedgerNotSet)?;

    let subaccount = get_deposit_subaccount(user, episode_id);
    let from_account = types::Account {
        owner: ic_cdk::api::id(),
        subaccount: Some(subaccount.to_vec()),
    };

    let balance_result: Result<(Nat,), _> =
        call(ledger_principal, "icrc1_balance_of", (from_account,)).await;

    let balance = match balance_result {
        Ok((balance,)) => balance,
        Err(_) => return Err(types::PoolError::LedgerCallFailed),
    };

    let transfer_amount = if balance > MINIMUM_DEPOSIT_AMOUNT.clone() {
        balance - TRANSFER_FEE.clone()
    } else {
        return Err(types::PoolError::InsufficientBalance);
    };

    let transfer_args = (types::TransferArg {
        from_subaccount: Some(subaccount.to_vec()),
        to: types::Account {
            owner: ic_cdk::api::id(),
            subaccount: None,
        },
        amount: transfer_amount.clone(),
        fee: Some(TRANSFER_FEE.clone()),
        memo: None,
        created_at_time: None,
    },);

    let transfer_result: Result<(Result<Nat, types::TransferError>,), _> =
        call(ledger_principal, "icrc1_transfer", transfer_args).await;

    if transfer_result.is_err() || transfer_result.as_ref().unwrap().0.is_err() {
        return Err(types::PoolError::TransferFailed);
    }

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

    let deposit = types::Deposit {
        episode: episode_id,
        shares: new_shares.clone(),
    };

    add_deposit(deposit_id, deposit, user, transfer_amount.clone(), true);

    Ok(())
}

#[ic_cdk::update]
pub async fn withdraw(deposit_id: u64) -> Result<(), types::PoolError> {
    let caller = ic_cdk::api::caller();
    let current_episode = get_current_episode();

    let deposit = DEPOSITS.with(|deposits| deposits.borrow().get(&deposit_id).clone());
    let deposit = match deposit {
        Some(d) => d,
        None => return Err(types::PoolError::NoDeposit),
    };

    let is_owner = USER_DEPOSITS.with(|user_deposits| {
        user_deposits
            .borrow()
            .get(&caller)
            .map(|deposits| deposits.0.contains(&deposit_id))
            .unwrap_or(false)
    });

    if !is_owner {
        return Err(types::PoolError::NotOwner);
    }

    if deposit.episode >= current_episode {
        return Err(types::PoolError::TimelockNotExpired);
    }

    let episode_data = EPISODES.with(|episodes| {
        episodes
            .borrow()
            .get(&deposit.episode)
            .ok_or(types::PoolError::NoDeposit)
    })?;

    let withdrawal_amount = deposit.shares.clone() * episode_data.assets_staked.clone()
        / episode_data.episode_shares.clone();

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

    let ledger_principal = TOKEN_ID.with(|cell| cell.borrow().get().clone());
    let transfer_amount = withdrawal_amount.clone() - TRANSFER_FEE.clone();
    let transfer_args = (types::TransferArg {
        from_subaccount: None,
        to: types::Account {
            owner: caller,
            subaccount: None,
        },
        amount: transfer_amount,
        fee: Some(TRANSFER_FEE.clone()),
        memo: None,
        created_at_time: None,
    },);
    let transfer_result: Result<(Result<Nat, types::TransferError>,), _> =
        call(ledger_principal, "icrc1_transfer", transfer_args).await;

    if transfer_result.is_err() || transfer_result.as_ref().unwrap().0.is_err() {
        add_deposit(
            deposit_id,
            deposit.clone(),
            caller,
            withdrawal_amount.clone(),
            false,
        );
        return Err(types::PoolError::TransferFailed);
    }

    Ok(())
}

ic_cdk::export_candid!();
