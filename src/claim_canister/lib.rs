use candid::{Nat, Principal};
use ic_cdk::api::call::call;
pub mod storage;
pub mod types;
use storage::*;
use types::*;


const TIMELOCK_DURATION: u64 = 24 * 60 * 60 * 1_000_000_000;

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

    USER_CLAIMS.with(|user_claims| {
        let mut user_claims_ref = user_claims.borrow_mut();
        let mut user_claims_list = user_claims_ref.get(&receiver).unwrap_or(UserClaims(vec![]));
        user_claims_list.0.push(claim_id);
        user_claims_ref.insert(receiver, user_claims_list);
    });

    Ok(claim_id)
}

#[ic_cdk::update]
pub fn approve_claim(claim_id: u64) -> Result<(), ClaimError> {
    let caller = ic_cdk::api::caller();
    
    let is_approver = APPROVERS.with(|approvers| {
        approvers.borrow().get(&caller).unwrap_or(false)
    });

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
    let claim = CLAIMS.with(|claims| {
        claims.borrow().get(&claim_id).ok_or(ClaimError::NotFound)
    })?;

    if claim.status != ClaimStatus::Approved {
        return Err(ClaimError::NotApproved);
    }

    let current_time = ic_cdk::api::time();
    let approved_time = claim.approved_at.ok_or(ClaimError::NotApproved)?;
    
    if current_time < approved_time + TIMELOCK_DURATION {
        return Err(ClaimError::TimelockNotExpired);
    }
        claim.pool_canister_id,
        "slash",
        (claim.receiver, claim.amount.clone()),
    ).await;

    if let Ok((Ok(()),)) = slash_result {
     
        CLAIMS.with(|claims| {
            let mut claims_ref = claims.borrow_mut();
            let mut updated_claim = claim.clone();
            updated_claim.status = ClaimStatus::Executed;
            claims_ref.insert(claim_id, updated_claim);
        });
        Ok(())
    } else {
        Err(ClaimError::PoolCallFailed)
    }
}

#[ic_cdk::query]
pub fn get_claim(claim_id: u64) -> Option<ClaimInfo> {
    CLAIMS.with(|claims| {
        claims.borrow().get(&claim_id).map(|claim| {
            let current_time = ic_cdk::api::time();
            let can_execute = if claim.status == ClaimStatus::Approved {
                if let Some(approved_time) = claim.approved_at {
                    current_time >= approved_time + TIMELOCK_DURATION
                } else {
                    false
                }
            } else {
                false
            };

            let time_until_execution = if claim.status == ClaimStatus::Approved {
                if let Some(approved_time) = claim.approved_at {
                    let execution_time = approved_time + TIMELOCK_DURATION;
                    if current_time < execution_time {
                        Some(execution_time - current_time)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            ClaimInfo {
                id: claim.id,
                receiver: claim.receiver,
                amount: claim.amount.clone(),
                pool_canister_id: claim.pool_canister_id,
                description: claim.description.clone(),
                status: claim.status.clone(),
                created_at: claim.created_at,
                approved_at: claim.approved_at,
                approved_by: claim.approved_by,
                can_execute,
                time_until_execution,
            }
        })
    })
}

#[ic_cdk::query]
pub fn get_user_claims(user: Principal) -> Vec<ClaimInfo> {
    let claim_ids = USER_CLAIMS.with(|user_claims| {
        user_claims.borrow().get(&user).map(|claims| claims.0.clone()).unwrap_or_default()
    });

    claim_ids.iter()
        .filter_map(|&claim_id| get_claim(claim_id))
        .collect()
}


#[ic_cdk::query]
pub fn is_approver(principal: Principal) -> bool {
    APPROVERS.with(|approvers| {
        approvers.borrow().get(&principal).unwrap_or(false)
    })
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


    if approver == owner {
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
