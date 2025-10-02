use crate::CanisterClient;
use candid::{Nat, Principal};

use pool_canister::{Account, TransferArg, TransferError};

#[derive(candid::CandidType, candid::Deserialize, Debug)]
pub enum TransferResult {
    Ok(Nat),
    Err(TransferError),
}

pub struct LedgerCanisterClient<'a> {
    pub client: CanisterClient<'a>,
}

impl<'a> LedgerCanisterClient<'a> {
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
        update icrc1_transfer(transfer_args: TransferArg) -> TransferResult;

        query icrc1_balance_of(account: Account) -> Nat;
        query icrc1_fee() -> Nat;
        query icrc1_metadata() -> Vec<(String, String)>;
        query icrc1_name() -> String;
        query icrc1_symbol() -> String;
        query icrc1_decimals() -> u8;
        query icrc1_total_supply() -> Nat;
    }
}
