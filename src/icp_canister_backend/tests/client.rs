#![allow(dead_code)]
#[path = "types.rs"]
mod types;
use candid::{decode_one, encode_args, Nat, Principal};
use icp_canister_backend::{Account, Deposit, Episode, PoolError, PoolState, UserDepositInfo};
use pocket_ic::PocketIc;
pub use types::TransferResult;

pub struct PoolCanister<'a> {
    pub pic: &'a PocketIc,
    pub canister_id: Principal,
    pub ledger_id: Principal,
    caller: Principal,
}

impl<'a> PoolCanister<'a> {
    pub fn new(pic: &'a PocketIc, canister_id: Principal, ledger_id: Principal) -> Self {
        Self {
            pic,
            canister_id,
            ledger_id,
            caller: Principal::anonymous(),
        }
    }

    pub fn connect(&mut self, caller_principal: Principal) -> &mut Self {
        self.caller = caller_principal;
        self
    }

    // Canister methods
    pub fn deposit(&self, user: Principal, episode: u64) -> Result<(), PoolError> {
        let result = self
            .pic
            .update_call(
                self.canister_id,
                self.caller,
                "deposit",
                encode_args((user, episode)).unwrap(),
            )
            .expect("Failed to call deposit");
        decode_one(&result).unwrap()
    }

    pub fn withdraw(&self, deposit_id: u64) -> Result<(), PoolError> {
        let result = self
            .pic
            .update_call(
                self.canister_id,
                self.caller,
                "withdraw",
                encode_args((deposit_id,)).unwrap(),
            )
            .expect("Failed to call withdraw");
        decode_one(&result).unwrap()
    }

    pub fn get_deposit(&self, deposit_id: u64) -> Option<Deposit> {
        let result = self
            .pic
            .query_call(
                self.canister_id,
                self.caller,
                "get_deposit",
                encode_args((deposit_id,)).unwrap(),
            )
            .expect("Failed to call get_deposit");
        decode_one(&result).unwrap()
    }

    pub fn get_user_deposits(&self, user: Principal) -> Vec<UserDepositInfo> {
        let result = self
            .pic
            .query_call(
                self.canister_id,
                self.caller,
                "get_user_deposits",
                encode_args((user,)).unwrap(),
            )
            .expect("Failed to call get_user_deposits");
        decode_one(&result).unwrap()
    }

    pub fn get_deposit_subaccount(&self, user: Principal, episode: u64) -> [u8; 32] {
        let result = self
            .pic
            .query_call(
                self.canister_id,
                self.caller,
                "get_deposit_subaccount",
                encode_args((user, episode)).unwrap(),
            )
            .expect("Failed to call get_deposit_subaccount");
        decode_one(&result).unwrap()
    }

    pub fn get_current_episode_id(&self) -> u64 {
        let result = self
            .pic
            .query_call(
                self.canister_id,
                self.caller,
                "get_current_episode_id",
                encode_args(()).unwrap(),
            )
            .expect("Failed to call get_current_episode_id");
        decode_one(&result).unwrap()
    }

    pub fn get_episode(&self, episode_id: u64) -> Option<Episode> {
        let result = self
            .pic
            .query_call(
                self.canister_id,
                self.caller,
                "get_episode",
                encode_args((episode_id,)).unwrap(),
            )
            .expect("Failed to call get_episode");
        decode_one(&result).unwrap()
    }

    pub fn get_pool_state(&self) -> PoolState {
        let result = self
            .pic
            .query_call(
                self.canister_id,
                self.caller,
                "get_pool_state",
                encode_args(()).unwrap(),
            )
            .expect("Failed to call get_pool_state");
        decode_one(&result).unwrap()
    }

    pub fn get_pool_reward_rate(&self) -> Nat {
        let result = self
            .pic
            .query_call(
                self.canister_id,
                self.caller,
                "get_pool_reward_rate",
                encode_args(()).unwrap(),
            )
            .expect("Failed to call get_pool_reward_rate");
        decode_one(&result).unwrap()
    }

    pub fn get_reward_subaccount(&self) -> [u8; 32] {
        let result = self
            .pic
            .query_call(
                self.canister_id,
                self.caller,
                "get_reward_subaccount",
                encode_args(()).unwrap(),
            )
            .expect("Failed to call get_reward_subaccount");
        decode_one(&result).unwrap()
    }

    pub fn reward_pool(&self) -> Result<(), PoolError> {
        let result = self
            .pic
            .update_call(
                self.canister_id,
                self.caller,
                "reward_pool",
                encode_args(()).unwrap(),
            )
            .expect("Failed to call reward_pool");
        decode_one(&result).unwrap()
    }

    pub fn set_executor_principal(&self, executor: Principal) -> Result<(), PoolError> {
        let result = self
            .pic
            .update_call(
                self.canister_id,
                self.caller,
                "set_executor_principal",
                encode_args((executor,)).unwrap(),
            )
            .expect("Failed to call set_executor_principal");
        decode_one(&result).unwrap()
    }

    pub fn slash(&self, receiver: Principal, amount: Nat) -> Result<(), PoolError> {
        let result = self
            .pic
            .update_call(
                self.canister_id,
                self.caller,
                "slash",
                encode_args((receiver, amount)).unwrap(),
            )
            .expect("Failed to call slash");
        decode_one(&result).unwrap()
    }

    pub fn update_episodes_state(&self) {
        self.pic
            .update_call(
                self.canister_id,
                self.caller,
                "update_episodes_state",
                encode_args(()).unwrap(),
            )
            .expect("Failed to call update_episodes_state");
    }

    pub fn get_deposits_rewards(&self, deposit_ids: Vec<u64>) -> Nat {
        let result = self
            .pic
            .query_call(
                self.canister_id,
                self.caller,
                "get_deposits_rewards",
                encode_args((deposit_ids,)).unwrap(),
            )
            .expect("Failed to call get_deposits_rewards");
        decode_one(&result).unwrap()
    }

    pub fn withdraw_rewards(&self, deposit_ids: Vec<u64>) -> Result<Nat, PoolError> {
        let result = self
            .pic
            .update_call(
                self.canister_id,
                self.caller,
                "withdraw_rewards",
                encode_args((deposit_ids,)).unwrap(),
            )
            .expect("Failed to call withdraw_rewards");
        decode_one(&result).unwrap()
    }

    // Ledger methods
    pub fn icrc1_balance_of(&self, account: Account) -> Nat {
        let result = self
            .pic
            .query_call(
                self.ledger_id,
                self.caller,
                "icrc1_balance_of",
                encode_args((account,)).unwrap(),
            )
            .expect("Failed to call icrc1_balance_of");
        decode_one(&result).unwrap()
    }

    pub fn icrc1_transfer(
        &self,
        transfer_args: icp_canister_backend::TransferArg,
    ) -> TransferResult {
        let result = self
            .pic
            .update_call(
                self.ledger_id,
                self.caller,
                "icrc1_transfer",
                encode_args((transfer_args,)).unwrap(),
            )
            .expect("Failed to call icrc1_transfer");
        decode_one(&result).unwrap()
    }
}
