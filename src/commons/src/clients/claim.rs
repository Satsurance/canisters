use crate::CanisterClient;
use candid::{Nat, Principal};
use claim_canister::types::{ClaimError, ClaimInfo};

pub struct ClaimCanisterClient<'a> {
    pub client: CanisterClient<'a>,
}

impl<'a> ClaimCanisterClient<'a> {
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
        update add_claim(receiver: Principal, amount: Nat, pool_canister: Principal, desc: String) -> Result<u64, ClaimError>;
        update approve_claim(claim_id: u64) -> Result<(), ClaimError>;
        update execute_claim(claim_id: u64) -> Result<(), ClaimError>;
        update add_approver(approver: Principal) -> Result<(), ClaimError>;
        update remove_approver(approver: Principal) -> Result<(), ClaimError>;
        update withdraw_deposit(claim_id: u64) -> Result<(), ClaimError>;
        update mark_as_spam(claim_id: u64) -> Result<(), ClaimError>;
        update set_claim_deposit(new_deposit: Nat) -> Result<(), ClaimError>;

        query get_claim(claim_id: u64) -> Option<ClaimInfo>;
        query is_approver(principal: Principal) -> bool;
        query get_claim_deposit() -> Nat;
        query get_claim_deposit_subaccount(user: Principal, receiver: Principal, amount: Nat, pool_canister_id: Principal, description: String) -> [u8; 32];
    }
}
