use candid::{Nat, Principal};
use ic_cdk::api::call::call;

pub mod claims;
pub mod governance;
pub mod storage;
pub mod types;

use storage::*;
use types::*;

lazy_static::lazy_static! {
    pub static ref TRANSFER_FEE: Nat = Nat::from(10u64);
}

// Utility functions
pub async fn get_subaccount_balance(subaccount: Vec<u8>) -> Result<Nat, ClaimError> {
    let ledger_id = LEDGER_CANISTER_ID.with(|cell| {
        let id = cell.borrow().get().clone();
        if id == Principal::anonymous() {
            return Err(ClaimError::LedgerNotSet);
        }
        Ok(id)
    })?;

    let account = Account {
        owner: ic_cdk::api::id(),
        subaccount: Some(subaccount),
    };

    let result: Result<(Nat,), _> = call(ledger_id, "icrc1_balance_of", (account,)).await;

    match result {
        Ok((balance,)) => Ok(balance),
        Err(_) => Err(ClaimError::DepositTransferFailed),
    }
}

pub async fn transfer_icrc1(
    from_subaccount: Option<Vec<u8>>,
    to: Principal,
    gross_amount: Nat,
) -> Result<(), ClaimError> {
    let ledger_id = LEDGER_CANISTER_ID.with(|cell| {
        let id = cell.borrow().get().clone();
        if id == Principal::anonymous() {
            return Err(ClaimError::LedgerNotSet);
        }
        Ok(id)
    })?;

    if gross_amount <= TRANSFER_FEE.clone() {
        return Ok(());
    }

    let net_amount = gross_amount - TRANSFER_FEE.clone();

    let transfer_args = TransferArg {
        from_subaccount,
        to: Account {
            owner: to,
            subaccount: None,
        },
        amount: net_amount,
        fee: Some(TRANSFER_FEE.clone()),
        memo: None,
        created_at_time: None,
    };

    let result: Result<(Result<Nat, TransferError>,), _> =
        call(ledger_id, "icrc1_transfer", (transfer_args,)).await;

    match result {
        Ok((Ok(_),)) => Ok(()),
        _ => Err(ClaimError::DepositTransferFailed),
    }
}

#[ic_cdk::init]
pub fn init(
    owner: Principal,
    claim_deposit: Nat,
    ledger_canister_id: Principal,
    approval_period: u64,
    execution_timeout: u64,
) {
    OWNER.with(|cell| {
        cell.borrow_mut().set(owner).ok();
    });

    APPROVERS.with(|approvers| {
        approvers.borrow_mut().insert(owner, true);
    });

    CLAIM_DEPOSIT.with(|cell| {
        cell.borrow_mut().set(StorableNat(claim_deposit)).ok();
    });

    LEDGER_CANISTER_ID.with(|cell| {
        cell.borrow_mut().set(ledger_canister_id).ok();
    });

    APPROVAL_PERIOD.with(|cell| {
        cell.borrow_mut().set(approval_period).ok();
    });

    EXECUTION_TIMEOUT.with(|cell| {
        cell.borrow_mut().set(execution_timeout).ok();
    });
}

ic_cdk::export_candid!();
