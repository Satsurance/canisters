use crate::types::{Claim, Memory, StorableNat};
use candid::{Nat, Principal};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, StableCell};
use std::cell::RefCell;

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    pub static CLAIM_COUNTER: RefCell<StableCell<u64, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
            0u64
        ).expect("Failed to initialize CLAIM_COUNTER")
    );

    pub static CLAIMS: RefCell<StableBTreeMap<u64, Claim, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))),
        )
    );

    pub static APPROVERS: RefCell<StableBTreeMap<Principal, bool, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3))),
        )
    );

    pub static OWNER: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4))),
            Principal::anonymous()
        ).expect("Failed to initialize OWNER")
    );

    pub static CLAIM_DEPOSIT: RefCell<StableCell<StorableNat, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(5))),
            StorableNat(Nat::from(0u64))
        ).expect("Failed to initialize CLAIM_DEPOSIT")
    );

    pub static LEDGER_CANISTER_ID: RefCell<StableCell<Principal, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(6))),
            Principal::anonymous()
        ).expect("Failed to initialize LEDGER_CANISTER_ID")
    );

    pub static APPROVAL_PERIOD: RefCell<StableCell<u64, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(7))),
            0u64
        ).expect("Failed to initialize APPROVAL_PERIOD")
    );
}
