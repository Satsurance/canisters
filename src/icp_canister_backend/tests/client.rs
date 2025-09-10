#![allow(dead_code)]
#[path = "types.rs"]
mod types;
use candid::{decode_one, encode_args, Nat, Principal};
use icp_canister_backend::{Account, Deposit, Episode, PoolError, PoolState, UserDepositInfo};
use pocket_ic::PocketIc;
pub use types::TransferResult;

pub struct Client<'a> {
    pub pic: &'a PocketIc,
    pub canister_id: Principal,
    pub ledger_id: Principal,
    caller: Principal,
}

impl<'a> Client<'a> {
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

    pub fn get_stakable_episode(&self, relative_episode: u8) -> u64 {
        if relative_episode > 7 {
            panic!("Relative episode must be 0-7");
        }

        let current_episode = self.get_current_episode_id();
        let mut first_stakable = current_episode;
        while first_stakable % 3 != 2 {
            first_stakable += 1;
        }
        first_stakable + (relative_episode as u64 * 3)
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

    pub fn create_deposit(&self, user: Principal, amount: Nat, episode: u64) {
        let subaccount = self.get_deposit_subaccount(user, episode);

        let transfer_args = icp_canister_backend::TransferArg {
            from_subaccount: None,
            to: Account {
                owner: self.canister_id,
                subaccount: Some(subaccount.to_vec()),
            },
            amount: amount.clone(),
            fee: Some(Nat::from(10_000u64)),
            memo: None,
            created_at_time: None,
        };

        let transfer_result = self.icrc1_transfer(transfer_args);
        assert!(matches!(transfer_result, TransferResult::Ok(_)), "Transfer should succeed");

        let result = self.deposit(user, episode);
        assert!(result.is_ok(), "Deposit should succeed: {:?}", result);
    }

    // Utility methods
    pub fn advance_time(&self, duration_seconds: u64) {
        self.pic
            .advance_time(std::time::Duration::from_secs(duration_seconds));
        self.pic.tick();
    }

    pub fn get_current_time(&self) -> u64 {
        self.pic.get_time().as_nanos_since_unix_epoch() / 1_000_000_000
    }

    pub fn get_episode_time_to_end(&self, target_episode: u64) -> u64 {
        let target_episode_end_time =
            (target_episode + 1) * icp_canister_backend::EPISODE_DURATION;
        let current_time = self.get_current_time();
        target_episode_end_time - current_time
    }
}
