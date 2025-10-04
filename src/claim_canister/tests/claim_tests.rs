mod setup;
use candid::{Nat, Principal};
use claim_canister::types::{ClaimError, ClaimStatus};
use claim_canister::TIMELOCK_DURATION;
use setup::setup;
use std::time::Duration;
use pool_canister::TRANSFER_FEE;
use commons::{ClaimCanisterClient, LedgerCanisterClient};

#[test]
fn test_claim_positive_flow() {
    let (pic, claim_canister, pool_canister, owner, ledger_id) = setup();

    let receiver_bytes = [3u8; 29];
    let receiver = Principal::from_slice(&receiver_bytes);
    let amount = Nat::from(1_000_000u64);
    let desc = String::from("Insurance payout for property damage");

    // Create claim client
    let mut claim_client = ClaimCanisterClient::new(&pic, claim_canister);

    // Add claim
    let claim_id = claim_client
        .connect(owner)
        .add_claim(receiver, amount.clone(), pool_canister, desc)
        .expect("add_claim should succeed");

    // Approve claim
    claim_client
        .connect(owner)
        .approve_claim(claim_id)
        .expect("approve_claim should succeed");

    // Verify claim is approved before timelock
    let claim_info = claim_client
        .connect(owner)
        .get_claim(claim_id)
        .expect("expected claim to exist after approval");
    assert_eq!(claim_info.status, ClaimStatus::Approved);

    let timelock_duration: Duration = Duration::from_nanos(TIMELOCK_DURATION);
    pic.advance_time(timelock_duration);

    // Get receiver's balance before execution
    let receiver_account = pool_canister::types::Account {
        owner: receiver,
        subaccount: None
    };
    let mut ledger_client = LedgerCanisterClient::new(&pic, ledger_id);
    let balance_before = ledger_client
        .connect(owner)
        .icrc1_balance_of(receiver_account.clone());

    // Execute claim
    claim_client
        .connect(receiver)
        .execute_claim(claim_id)
        .expect("execute_claim should succeed");

    // Verify claim status is Executed
    let final_info = claim_client
        .connect(owner)
        .get_claim(claim_id)
        .expect("expected claim to exist after execution");
    assert_eq!(final_info.status, ClaimStatus::Executed);

    // Verify receiver received the expected amount
    let balance_after = ledger_client
        .connect(owner)
        .icrc1_balance_of(receiver_account);

    let expected_balance = balance_before.clone() + amount.clone() - TRANSFER_FEE.clone();
    assert_eq!(balance_after, expected_balance,
        "Receiver balance should increase by the claim amount minus transfer fee. Before: {}, After: {}, Expected: {}",
        balance_before, balance_after, expected_balance);
}

#[test]
fn test_execute_only_after_timelock() {
    let (pic, claim_canister, pool_canister, owner, _ledger_id) = setup();

    let receiver_bytes = [3u8; 29];
    let receiver = Principal::from_slice(&receiver_bytes);
    let amount = Nat::from(1_000_000u64);
    let desc = String::from("Timelock test");

    let mut claim_client = ClaimCanisterClient::new(&pic, claim_canister);

    // Add claim
    let claim_id = claim_client
        .connect(owner)
        .add_claim(receiver, amount, pool_canister, desc)
        .expect("add_claim should succeed");

    // Approve claim
    claim_client
        .connect(owner)
        .approve_claim(claim_id)
        .expect("approve_claim should succeed");

    // Try to execute before timelock - should fail
    let exec_result = claim_client
        .connect(receiver)
        .execute_claim(claim_id);
    assert_eq!(exec_result, Err(ClaimError::TimelockNotExpired));

    // After timelock
    let timelock_duration: Duration = Duration::from_nanos(TIMELOCK_DURATION);
    pic.advance_time(timelock_duration);

    // Execute claim - should succeed
    claim_client
        .connect(receiver)
        .execute_claim(claim_id)
        .expect("execute_claim should succeed after timelock");

    // Verify claim status is Executed
    let final_info = claim_client
        .connect(owner)
        .get_claim(claim_id)
        .expect("expected claim to exist after execution");
    assert_eq!(final_info.status, ClaimStatus::Executed);
}

#[test]
fn test_cannot_execute_same_claim_multiple_times() {
    let (pic, claim_canister, pool_canister, owner, _ledger_id) = setup();

    let receiver_bytes = [4u8; 29];
    let receiver = Principal::from_slice(&receiver_bytes);
    let amount = Nat::from(1_000_000u64);
    let desc = String::from("Double execute test");

    let mut claim_client = ClaimCanisterClient::new(&pic, claim_canister);

    // Create and approve claim
    let claim_id = claim_client
        .connect(owner)
        .add_claim(receiver, amount, pool_canister, desc)
        .expect("add_claim should succeed");

    claim_client
        .connect(owner)
        .approve_claim(claim_id)
        .expect("approve_claim should succeed");

    // Advance time past timelock
    pic.advance_time(Duration::from_nanos(TIMELOCK_DURATION));

    // First execution - should succeed
    let first_result = claim_client
        .connect(receiver)
        .execute_claim(claim_id);
    assert_eq!(first_result, Ok(()));

    let claim_info1 = claim_client
        .connect(owner)
        .get_claim(claim_id)
        .expect("claim should exist");
    assert_eq!(claim_info1.status, ClaimStatus::Executed);

    // Second execution - should fail
    let second_result = claim_client
        .connect(receiver)
        .execute_claim(claim_id);
    assert_eq!(second_result, Err(ClaimError::AlreadyExecuted));

    let claim_info2 = claim_client
        .connect(owner)
        .get_claim(claim_id)
        .expect("claim should still exist");
    assert_eq!(claim_info2.status, ClaimStatus::Executed);
}

#[test]
fn test_execute_before_approval_not_possible() {
    let (pic, claim_canister, pool_canister, owner, _ledger_id) = setup();

    let receiver_bytes = [5u8; 29];
    let receiver = Principal::from_slice(&receiver_bytes);
    let amount = Nat::from(777u64);
    let desc = String::from("Execute before approval");

    let mut claim_client = ClaimCanisterClient::new(&pic, claim_canister);

    // Create claim but don't approve
    let claim_id = claim_client
        .connect(owner)
        .add_claim(receiver, amount, pool_canister, desc)
        .expect("add_claim should succeed");

    // Try to execute without approval - should fail
    let exec_result = claim_client
        .connect(receiver)
        .execute_claim(claim_id);
    assert_eq!(exec_result, Err(ClaimError::NotApproved));
}

#[test]
fn test_only_owner_can_change_approvers() {
    let (pic, claim_canister, _pool_canister, owner, _ledger_id) = setup();

    let other_bytes = [9u8; 29];
    let other = Principal::from_slice(&other_bytes);

    let mut claim_client = ClaimCanisterClient::new(&pic, claim_canister);

    // Non-owner cannot add approver
    let add_result_non_owner = claim_client
        .connect(other)
        .add_approver(other);
    assert_eq!(add_result_non_owner, Err(ClaimError::InsufficientPermissions));

    // Owner can add approver
    let add_result_owner = claim_client
        .connect(owner)
        .add_approver(other);
    assert_eq!(add_result_owner, Ok(()));

    // Non-owner cannot remove approver
    let remove_result_non_owner = claim_client
        .connect(other)
        .remove_approver(other);
    assert_eq!(remove_result_non_owner, Err(ClaimError::InsufficientPermissions));

    // Owner can remove approver
    let remove_result_owner = claim_client
        .connect(owner)
        .remove_approver(other);
    assert_eq!(remove_result_owner, Ok(()));

    // Verify the approver was actually removed
    let is_approver = claim_client
        .connect(owner)
        .is_approver(other);
    assert!(!is_approver, "Approver should be removed and no longer be able to approve");
}

#[test]
fn test_claim_status_reverts_to_approved_on_slash_failure() {
    let (pic, claim_canister, pool_canister, owner, _ledger_id) = setup();

    let receiver_bytes = [6u8; 29];
    let receiver = Principal::from_slice(&receiver_bytes);
    let amount = Nat::from(2000u64);
    let desc = String::from("Slash failure test");

    let mut claim_client = ClaimCanisterClient::new(&pic, claim_canister);

    // Create and approve claim
    let claim_id = claim_client
        .connect(owner)
        .add_claim(receiver, amount, pool_canister, desc)
        .expect("add_claim should succeed");

    claim_client
        .connect(owner)
        .approve_claim(claim_id)
        .expect("approve_claim should succeed");


    let timelock_duration: Duration = Duration::from_nanos(TIMELOCK_DURATION);
    pic.advance_time(timelock_duration);

    // Execute claim (this will fail due to insufficient balance in pool)
    let exec_result = claim_client
        .connect(receiver)
        .execute_claim(claim_id);
    assert_eq!(
        exec_result,
        Err(ClaimError::PoolCallFailed(
            "Ok((Err(InsufficientBalance),))".to_string()
        ))
    );

    // Verify claim status reverted back to Approved
    let claim_info = claim_client
        .connect(owner)
        .get_claim(claim_id)
        .expect("expected claim to exist after failed slash");
    assert_eq!(claim_info.status, ClaimStatus::Approved);
}
