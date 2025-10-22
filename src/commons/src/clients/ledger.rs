use crate::CanisterClient;
use candid::{Nat, Principal};

use pool_canister::{Account, TransferArg, TransferError};

#[derive(candid::CandidType, candid::Deserialize, Debug)]
pub enum TransferResult {
    Ok(Nat),
    Err(TransferError),
}

#[derive(candid::CandidType, candid::Deserialize, Debug)]
pub struct ApproveArgs {
    pub from_subaccount: Option<Vec<u8>>,
    pub spender: Account,
    pub amount: Nat,
    pub expected_allowance: Option<Nat>,
    pub expires_at: Option<u64>,
    pub fee: Option<Nat>,
    pub memo: Option<Vec<u8>>,
    pub created_at_time: Option<u64>,
}

#[derive(candid::CandidType, candid::Deserialize, Debug)]
pub enum ApproveError {
    BadFee { expected_fee: Nat },
    InsufficientFunds { balance: Nat },
    AllowanceChanged { current_allowance: Nat },
    Expired { ledger_time: u64 },
    TooOld,
    CreatedInFuture { ledger_time: u64 },
    Duplicate { duplicate_of: Nat },
    TemporarilyUnavailable,
    GenericError { error_code: Nat, message: String },
}

#[derive(candid::CandidType, candid::Deserialize, Debug)]
pub enum ApproveResult {
    Ok(Nat),
    Err(ApproveError),
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
        update icrc2_approve(args: ApproveArgs) -> ApproveResult;

        query icrc1_balance_of(account: Account) -> Nat;
        query icrc1_fee() -> Nat;
        query icrc1_metadata() -> Vec<(String, String)>;
        query icrc1_name() -> String;
        query icrc1_symbol() -> String;
        query icrc1_decimals() -> u8;
        query icrc1_total_supply() -> Nat;
    }
}
