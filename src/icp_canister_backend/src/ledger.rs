use crate::storage::TOKEN_ID;
use crate::types::{Account, PoolError, TransferArg, TransferError};
use crate::TRANSFER_FEE;
use candid::{Nat, Principal};
use ic_cdk::api::call::call;
use sha2::{Digest, Sha256};

#[ic_cdk::query]
pub fn get_deposit_subaccount(user: Principal, episode: u64) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(user.as_slice());
    hasher.update(episode.to_be_bytes());
    hasher.finalize().into()
}

#[ic_cdk::query]
pub fn get_reward_subaccount() -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"REWARD_SUBACCOUNT");
    hasher.finalize().into()
}

fn get_ledger_principal() -> Result<Principal, PoolError> {
    TOKEN_ID.with(|cell| {
        let stored = cell.borrow().get().clone();
        if stored == Principal::anonymous() {
            Err(PoolError::LedgerNotSet)
        } else {
            Ok(stored)
        }
    })
}

pub async fn get_subaccount_balance(subaccount: Vec<u8>) -> Result<Nat, PoolError> {
    let ledger_principal = get_ledger_principal()?;

    let account = Account {
        owner: ic_cdk::api::id(),
        subaccount: Some(subaccount),
    };

    let balance_result: Result<(Nat,), _> =
        call(ledger_principal, "icrc1_balance_of", (account,)).await;

    match balance_result {
        Ok((balance,)) => Ok(balance),
        Err(_) => Err(PoolError::LedgerCallFailed),
    }
}

pub async fn transfer_icrc1(
    from_subaccount: Option<Vec<u8>>,
    to: Principal,
    gross_amount: Nat,
) -> Result<Nat, PoolError> {
    if gross_amount <= TRANSFER_FEE.clone() {
        return Err(PoolError::InsufficientBalance);
    }
    let net_amount = gross_amount - TRANSFER_FEE.clone();

    let ledger_principal = get_ledger_principal()?;

    let transfer_args = (TransferArg {
        from_subaccount,
        to: Account {
            owner: to,
            subaccount: None,
        },
        amount: net_amount,
        fee: Some(TRANSFER_FEE.clone()),
        memo: None,
        created_at_time: None,
    },);

    let transfer_result: Result<(Result<Nat, TransferError>,), _> =
        call(ledger_principal, "icrc1_transfer", transfer_args).await;

    match transfer_result {
        Ok((Ok(block_index),)) => Ok(block_index),
        _ => Err(PoolError::TransferFailed),
    }
}
