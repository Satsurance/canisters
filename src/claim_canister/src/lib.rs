use candid::{Nat, Principal};
use ic_cdk::api::call::call;
use sha2::{Digest, Sha256};
pub mod storage;
pub mod types;
use storage::*;
use types::*;

lazy_static::lazy_static! {
    pub static ref TRANSFER_FEE: Nat = Nat::from(10u64);
}

async fn get_subaccount_balance(subaccount: Vec<u8>) -> Result<Nat, ClaimError> {
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

async fn transfer_icrc1(
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

#[ic_cdk::query]
pub fn get_claim_deposit_subaccount(
    user: Principal,
    receiver: Principal,
    amount: Nat,
    pool_canister_id: Principal,
    description: String,
) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(user.as_slice());
    hasher.update(receiver.as_slice());
    hasher.update(&candid::encode_one(&amount).unwrap());
    hasher.update(pool_canister_id.as_slice());
    hasher.update(description.as_bytes());
    hasher.finalize().into()
}

#[ic_cdk::query]
pub fn get_claim_deposit() -> Nat {
    CLAIM_DEPOSIT.with(|cell| cell.borrow().get().clone().0)
}

#[ic_cdk::query]
pub fn get_execution_timeout() -> u64 {
    EXECUTION_TIMEOUT.with(|cell| cell.borrow().get().clone())
}

#[ic_cdk::query]
pub fn get_claim(claim_id: u64) -> Option<ClaimInfo> {
    CLAIMS.with(|claims| {
        claims.borrow().get(&claim_id).map(|claim| ClaimInfo {
            id: claim.id,
            proposer: claim.proposer,
            receiver: claim.receiver,
            amount: claim.amount.clone(),
            pool_canister_id: claim.pool_canister_id,
            description: claim.description.clone(),
            status: claim.status.clone(),
            created_at: claim.created_at,
            approved_at: claim.approved_at,
            approved_by: claim.approved_by,
            deposit_amount: claim.deposit_amount.clone(),
        })
    })
}

#[ic_cdk::query]
pub fn is_approver(principal: Principal) -> bool {
    APPROVERS.with(|approvers| approvers.borrow().get(&principal).unwrap_or(false))
}

#[ic_cdk::query]
pub fn get_owner() -> Principal {
    OWNER.with(|cell| cell.borrow().get().clone())
}

#[ic_cdk::query]
pub fn get_next_claim_id() -> u64 {
    CLAIM_COUNTER.with(|counter| counter.borrow().get().clone() + 1)
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

#[ic_cdk::update]
pub async fn add_claim(
    receiver: Principal,
    amount: Nat,
    pool_canister_id: Principal,
    description: String,
) -> Result<u64, ClaimError> {
    let caller = ic_cdk::api::caller();

    let required_deposit = CLAIM_DEPOSIT.with(|cell| cell.borrow().get().clone().0);

    if required_deposit > Nat::from(0u64) {
        let subaccount = get_claim_deposit_subaccount(
            caller,
            receiver,
            amount.clone(),
            pool_canister_id,
            description.clone(),
        );
        let balance = get_subaccount_balance(subaccount.to_vec()).await?;

        if balance < required_deposit {
            return Err(ClaimError::InsufficientDeposit);
        }
    }

    let claim_id = CLAIM_COUNTER.with(|counter| {
        let current = counter.borrow().get().clone();
        let new_counter = current + 1;
        counter.borrow_mut().set(new_counter).ok();
        new_counter
    });

    let current_time = ic_cdk::api::time();

    let claim = Claim {
        id: claim_id,
        proposer: caller,
        receiver,
        amount: amount.clone(),
        pool_canister_id,
        description,
        status: ClaimStatus::Pending,
        created_at: current_time,
        approved_at: None,
        approved_by: None,
        deposit_amount: required_deposit,
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

    let approval_period = APPROVAL_PERIOD.with(|cell| cell.borrow().get().clone());

    CLAIMS.with(|claims| {
        let mut claims_ref = claims.borrow_mut();
        let mut claim = claims_ref.get(&claim_id).ok_or(ClaimError::NotFound)?;

        if claim.status == ClaimStatus::Approved {
            return Err(ClaimError::AlreadyApproved);
        }

        if claim.status == ClaimStatus::Executed {
            return Err(ClaimError::AlreadyExecuted);
        }

        let current_time = ic_cdk::api::time();
        if current_time > claim.created_at + approval_period {
            return Err(ClaimError::ApprovalPeriodExpired);
        }

        claim.status = ClaimStatus::Approved;
        claim.approved_at = Some(current_time);
        claim.approved_by = Some(caller);

        claims_ref.insert(claim_id, claim);
        Ok(())
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
            let execution_timeout = EXECUTION_TIMEOUT.with(|cell| cell.borrow().get().clone());

            if current_time < approved_time + execution_timeout {
                return Err(ClaimError::ExecutionTimeoutNotExpired);
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

        return Err(ClaimError::PoolCallFailed(format!("{:?}", slash_result)));
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

#[ic_cdk::update]
pub async fn withdraw_deposit(claim_id: u64) -> Result<(), ClaimError> {
    let caller = ic_cdk::api::caller();

    let (proposer, receiver, amount, pool_canister_id, description, deposit_amount) =
        CLAIMS.with(|claims| {
            let mut claims_ref = claims.borrow_mut();
            let mut claim = claims_ref.get(&claim_id).ok_or(ClaimError::NotFound)?;

            if claim.proposer != caller {
                return Err(ClaimError::NotProposer);
            }

            if claim.status == ClaimStatus::Pending
                && claim.created_at + APPROVAL_PERIOD.with(|cell| cell.borrow().get().clone())
                    > ic_cdk::api::time()
            {
                return Err(ClaimError::ApprovalPeriodNotExpired);
            }

            if claim.status == ClaimStatus::Approved
                && claim.approved_at.unwrap()
                    + EXECUTION_TIMEOUT.with(|cell| cell.borrow().get().clone())
                    > ic_cdk::api::time()
            {
                return Err(ClaimError::AlreadyApproved);
            }

            if claim.status == ClaimStatus::Spam {
                return Err(ClaimError::AlreadyMarkedAsSpam);
            }

            if claim.deposit_amount == Nat::from(0u64) {
                return Err(ClaimError::NoDepositToWithdraw);
            }

            let deposit_to_withdraw = claim.deposit_amount.clone();
            claim.deposit_amount = Nat::from(0u64);
            claims_ref.insert(claim_id, claim.clone());

            Ok((
                claim.proposer,
                claim.receiver,
                claim.amount.clone(),
                claim.pool_canister_id,
                claim.description.clone(),
                deposit_to_withdraw,
            ))
        })?;

    let subaccount =
        get_claim_deposit_subaccount(proposer, receiver, amount, pool_canister_id, description);
    transfer_icrc1(Some(subaccount.to_vec()), proposer, deposit_amount).await?;

    Ok(())
}

#[ic_cdk::update]
pub fn mark_as_spam(claim_id: u64) -> Result<(), ClaimError> {
    let caller = ic_cdk::api::caller();

    let is_approver = APPROVERS.with(|approvers| approvers.borrow().get(&caller).unwrap_or(false));

    if !is_approver {
        return Err(ClaimError::NotApprover);
    }

    CLAIMS.with(|claims| {
        let mut claims_ref = claims.borrow_mut();
        let mut claim = claims_ref.get(&claim_id).ok_or(ClaimError::NotFound)?;

        if claim.status == ClaimStatus::Executed {
            return Err(ClaimError::AlreadyExecuted);
        }

        if claim.status == ClaimStatus::Spam {
            return Err(ClaimError::AlreadyMarkedAsSpam);
        }

        claim.status = ClaimStatus::Spam;
        claims_ref.insert(claim_id, claim);

        Ok(())
    })
}

ic_cdk::export_candid!();
