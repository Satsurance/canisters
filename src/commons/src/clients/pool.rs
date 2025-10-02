use crate::CanisterClient;
use candid::{Nat, Principal};

use pool_canister::{Deposit, Episode, PoolError, PoolState, UserDepositInfo};

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
        update update_episodes_state() -> ();
        update withdraw_rewards(deposit_ids: Vec<u64>) -> Result<Nat, PoolError>;

        query get_deposit(deposit_id: u64) -> Option<Deposit>;
        query get_user_deposits(user: Principal) -> Vec<UserDepositInfo>;
        query get_deposit_subaccount(user: Principal, episode: u64) -> [u8; 32];
        query get_current_episode_id() -> u64;
        query get_episode(episode_id: u64) -> Option<Episode>;
        query get_pool_state() -> PoolState;
        query get_pool_reward_rate() -> Nat;
        query get_reward_subaccount() -> [u8; 32];
        query get_deposits_rewards(deposit_ids: Vec<u64>) -> Nat;
    }
}
