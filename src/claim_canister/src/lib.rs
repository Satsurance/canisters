use candid::{Nat, Principal};
use ic_cdk::api::call::call;
pub mod storage;
pub mod types;
use storage::*;
use types::*;

pub const TIMELOCK_DURATION: u64 = 24 * 60 * 60 * 1_000_000_000;

#[ic_cdk::init]
pub fn init(owner: Principal) {
    OWNER.with(|cell| {
        cell.borrow_mut().set(owner).ok();
    });

    APPROVERS.with(|approvers| {
        approvers.borrow_mut().insert(owner, true);
    });
}

#[ic_cdk::update]
pub fn add_claim(
    receiver: Principal,
    amount: Nat,
    pool_canister_id: Principal,
    description: String,
) -> Result<u64, ClaimError> {
    let claim_id = CLAIM_COUNTER.with(|counter| {
        let current = counter.borrow().get().clone();
        let new_counter = current + 1;
        counter.borrow_mut().set(new_counter).ok();
        new_counter
    });

    let current_time = ic_cdk::api::time();

    let claim = Claim {
        id: claim_id,
        receiver,
        amount: amount.clone(),
        pool_canister_id,
        description,
        status: ClaimStatus::Pending,
        created_at: current_time,
        approved_at: None,
        approved_by: None,
    };

    CLAIMS.with(|claims| {
        claims.borrow_mut().insert(claim_id, claim);
    });

    Ok(claim_id)
}

#[ic_cdk::update]
pub fn approve_claim(claim_id: u64) -> Result<(), ClaimError> {
    let caller = ic_cdk::api::caller();

    let is_approver = APPROVERS.with(|approvers| approvers.borrow().get(&caller).unwrap_or(false));

    if !is_approver {
        return Err(ClaimError::NotApprover);
    }

    CLAIMS.with(|claims| {
        let mut claims_ref = claims.borrow_mut();
        let mut claim = claims_ref.get(&claim_id).ok_or(ClaimError::NotFound)?;

        if claim.status == ClaimStatus::Pending {
            claim.status = ClaimStatus::Approved;
            claim.approved_at = Some(ic_cdk::api::time());
            claim.approved_by = Some(caller);
            claims_ref.insert(claim_id, claim);
            Ok(())
        } else if claim.status == ClaimStatus::Approved {
            Err(ClaimError::AlreadyApproved)
        } else if claim.status == ClaimStatus::Executed {
            Err(ClaimError::AlreadyExecuted)
        } else {
            Err(ClaimError::InvalidStatus)
        }
    })
}

#[ic_cdk::update]
pub async fn execute_claim(claim_id: u64) -> Result<(), ClaimError> {
    let (pool_canister_id, receiver, amount) = CLAIMS.with(
        |claims| -> Result<(Principal, Principal, Nat), ClaimError> {
            let mut claims_ref = claims.borrow_mut();
            let mut claim = claims_ref.get(&claim_id).ok_or(ClaimError::NotFound)?;

            if claim.status == ClaimStatus::Executed {
                return Err(ClaimError::AlreadyExecuted);
            }
            if claim.status != ClaimStatus::Approved {
                return Err(ClaimError::NotApproved);
            }

            let current_time = ic_cdk::api::time();
            let approved_time = claim.approved_at.ok_or(ClaimError::NotApproved)?;

            if current_time < approved_time + TIMELOCK_DURATION {
                return Err(ClaimError::TimelockNotExpired);
            }

            claim.status = ClaimStatus::Executing;
            let pool_canister_id = claim.pool_canister_id;
            let receiver = claim.receiver;
            let amount = claim.amount.clone();

            claims_ref.insert(claim_id, claim);
            Ok((pool_canister_id, receiver, amount))
        },
    )?;

    let slash_result: Result<(Result<(), PoolError>,), _> =
        call(pool_canister_id, "slash", (receiver, amount)).await;

    let success = matches!(slash_result, Ok((Ok(()),)));

    if !success {
        CLAIMS.with(|claims| {
            let mut claims_ref = claims.borrow_mut();
            if let Some(mut updated_claim) = claims_ref.get(&claim_id) {
                updated_claim.status = ClaimStatus::Approved;
                claims_ref.insert(claim_id, updated_claim);
            }
        });

        return Err(ClaimError::PoolCallFailed(format!(
            "{:?}",
            slash_result.unwrap().0.unwrap_err()
        )));
    }

    CLAIMS.with(|claims| {
        let mut claims_ref = claims.borrow_mut();
        if let Some(mut updated_claim) = claims_ref.get(&claim_id) {
            updated_claim.status = ClaimStatus::Executed;
            claims_ref.insert(claim_id, updated_claim);
        }
    });
    Ok(())
}

#[ic_cdk::query]
pub fn get_claim(claim_id: u64) -> Option<ClaimInfo> {
    CLAIMS.with(|claims| {
        claims.borrow().get(&claim_id).map(|claim| ClaimInfo {
            id: claim.id,
            receiver: claim.receiver,
            amount: claim.amount.clone(),
            pool_canister_id: claim.pool_canister_id,
            description: claim.description.clone(),
            status: claim.status.clone(),
            created_at: claim.created_at,
            approved_at: claim.approved_at,
            approved_by: claim.approved_by,
        })
    })
}

#[ic_cdk::query]
pub fn is_approver(principal: Principal) -> bool {
    APPROVERS.with(|approvers| approvers.borrow().get(&principal).unwrap_or(false))
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

#[ic_cdk::query]
pub fn get_owner() -> Principal {
    OWNER.with(|cell| cell.borrow().get().clone())
}

ic_cdk::export_candid!();
