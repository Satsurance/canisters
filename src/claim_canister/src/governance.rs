use candid::{Nat, Principal};

use crate::storage::*;
use crate::types::*;

#[ic_cdk::query]
pub fn is_approver(principal: Principal) -> bool {
    APPROVERS.with(|approvers| approvers.borrow().get(&principal).unwrap_or(false))
}

#[ic_cdk::query]
pub fn get_owner() -> Principal {
    OWNER.with(|cell| cell.borrow().get().clone())
}

#[ic_cdk::update]
pub fn set_claim_deposit(new_deposit: Nat) -> Result<(), ClaimError> {
    let caller = ic_cdk::api::caller();
    let owner = OWNER.with(|cell| cell.borrow().get().clone());

    if caller != owner {
        return Err(ClaimError::InsufficientPermissions);
    }

    CLAIM_DEPOSIT.with(|cell| {
        cell.borrow_mut().set(StorableNat(new_deposit)).ok();
    });

    Ok(())
}

#[ic_cdk::update]
pub fn set_execution_timeout(new_timeout: u64) -> Result<(), ClaimError> {
    let caller = ic_cdk::api::caller();
    let owner = OWNER.with(|cell| cell.borrow().get().clone());

    if caller != owner {
        return Err(ClaimError::InsufficientPermissions);
    }

    EXECUTION_TIMEOUT.with(|cell| {
        cell.borrow_mut().set(new_timeout).ok();
    });

    Ok(())
}

#[ic_cdk::update]
pub fn add_approver(approver: Principal) -> Result<(), ClaimError> {
    let caller = ic_cdk::api::caller();
    let owner = OWNER.with(|cell| cell.borrow().get().clone());

    if caller != owner {
        return Err(ClaimError::InsufficientPermissions);
    }

    APPROVERS.with(|approvers| {
        approvers.borrow_mut().insert(approver, true);
    });

    Ok(())
}

#[ic_cdk::update]
pub fn remove_approver(approver: Principal) -> Result<(), ClaimError> {
    let caller = ic_cdk::api::caller();
    let owner = OWNER.with(|cell| cell.borrow().get().clone());

    if caller != owner {
        return Err(ClaimError::InsufficientPermissions);
    }

    APPROVERS.with(|approvers| {
        approvers.borrow_mut().remove(&approver);
    });

    Ok(())
}
