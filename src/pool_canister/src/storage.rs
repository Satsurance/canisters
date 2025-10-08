use candid::{Nat, Principal};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, StableCell};
use std::cell::RefCell;

use crate::types::{Deposit, Episode, PoolState, Product, StorableNat, UserDeposits};

pub type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    pub static TOKEN_ID: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
            Principal::anonymous()
        ).expect("Failed to initialize StableCell")
    );

    pub static DEPOSIT_COUNTER: RefCell<StableCell<u64, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))),
            0u64
        ).expect("Failed to initialize StableCell")
    );

    pub static DEPOSITS: RefCell<StableBTreeMap<u64, Deposit, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2))),
        )
    );

    pub static USER_DEPOSITS: RefCell<StableBTreeMap<Principal, UserDeposits, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3))),
        )
    );

    pub static POOL_STATE: RefCell<StableCell<PoolState, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4))),
            PoolState {
                total_assets: Nat::from(0u64),
                total_shares: Nat::from(0u64),
            }
        ).expect("Failed to initialize PoolState")
    );

    pub static EPISODES: RefCell<StableBTreeMap<u64, Episode, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(5))),
        )
    );

    pub static LAST_TIME_UPDATED: RefCell<StableCell<u64, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(6))),
            0u64
        ).expect("Failed to initialize LAST_TIME_UPDATED")
    );

    pub static EXECUTOR_PRINCIPAL: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(7))),
            Principal::anonymous()
        ).expect("Failed to initialize EXECUTOR_PRINCIPAL")
    );

    pub static POOL_REWARD_RATE: RefCell<StableCell<StorableNat, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(8))),
            StorableNat(Nat::from(0u64))
        ).expect("Failed to initialize POOL_REWARD_RATE")
    );

    pub static ACCUMULATED_REWARD_PER_SHARE: RefCell<StableCell<StorableNat, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(9))),
            StorableNat(Nat::from(0u64))
        ).expect("Failed to initialize ACCUMULATED_REWARD_PER_SHARE")
    );

    pub static PRODUCT_COUNTER: RefCell<StableCell<u64, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(10))),
            0u64
        ).expect("Failed to initialize PRODUCT_COUNTER")
    );

    pub static PRODUCTS: RefCell<StableBTreeMap<u64, Product, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(11))),
        )
    );

    pub static EPISODE_ALLOCATION_CUT: RefCell<StableBTreeMap<(u64, u64), StorableNat, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(12))),
        )
    );

    pub static TOTAL_COVER_ALLOCATION: RefCell<StableCell<StorableNat, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(13))),
            StorableNat(Nat::from(0u64))
        ).expect("Failed to initialize TOTAL_COVER_ALLOCATION")
    );

    pub static POOL_MANAGER_PRINCIPAL: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(14))),
            Principal::anonymous()
        ).expect("Failed to initialize POOL_MANAGER_PRINCIPAL")
    );
}
