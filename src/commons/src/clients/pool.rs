use crate::CanisterClient;
use candid::{Nat, Principal};

use pool_canister::{Coverage, Deposit, Episode, PoolError, PoolState, Product, UserDepositInfo};


pub struct PoolCanisterClient<'a> {
    pub client: CanisterClient<'a>,
}

impl<'a> PoolCanisterClient<'a> {
    pub fn new(pic: &'a pocket_ic::PocketIc, canister_id: Principal) -> Self {
        Self {
            client: CanisterClient::new(pic, canister_id),
        }
    }

    pub fn connect(&mut self, caller: Principal) -> &mut Self {
        self.client.set_caller(caller);
        self
    }

    crate::canister_methods! {
        update deposit(user: Principal, episode: u64) -> Result<(), PoolError>;
        update withdraw(deposit_id: u64) -> Result<(), PoolError>;
        update slash(receiver: Principal, amount: Nat) -> Result<(), PoolError>;
        update reward_pool() -> Result<(), PoolError>;
        update set_executor_principal(executor: Principal) -> Result<(), PoolError>;
        update set_pool_manager_principal(pool_manager: Principal) -> Result<(), PoolError>;
        update update_episodes_state() -> ();
        update withdraw_rewards(deposit_ids: Vec<u64>) -> Result<Nat, PoolError>;
        update create_product(name: String, annual_percent: u64, max_coverage_duration: u64, max_pool_allocation_percent: u64) -> Result<u64, PoolError>;
        update set_product(product_id: u64, annual_percent: u64, max_coverage_duration: u64, max_pool_allocation_percent: u64, active: bool) -> Result<(), PoolError>;
        update purchase_coverage(product_id: u64, covered_account: Principal, coverage_duration: u64, coverage_amount: Nat) -> Result<(), PoolError>;

        query get_deposit(deposit_id: u64) -> Option<Deposit>;
        query get_user_deposits(user: Principal) -> Vec<UserDepositInfo>;
        query get_deposit_subaccount(user: Principal, episode: u64) -> [u8; 32];
        query get_purchase_subaccount(user: Principal, product_id: u64) -> [u8; 32];
        query get_current_episode_id() -> u64;
        query get_episode(episode_id: u64) -> Option<Episode>;
        query get_pool_state() -> PoolState;
        query get_pool_reward_rate() -> Nat;
        query get_reward_subaccount() -> [u8; 32];
        query get_deposits_rewards(deposit_ids: Vec<u64>) -> Nat;
        query get_products() -> Vec<Product>;
        query get_total_cover_allocation() -> Nat;
        query get_coverages(user: Principal) -> Vec<Coverage>;
        query get_coverage(coverage_id: u64) -> Option<Coverage>;
    }
}
