mod setup;
mod utils;
use candid::{Nat, Principal};
use claim_canister::types::{ClaimError, ClaimStatus};
use commons::{ClaimCanisterClient, LedgerCanisterClient};
use pool_canister::TRANSFER_FEE;
use setup::setup;
use std::time::Duration;
use utils::transfer_to_deposit_subaccount;

// Constants
const APPROVAL_PERIOD_NANOS: u64 = 7 * 24 * 60 * 60 * 1_000_000_000; // 7 days in nanoseconds
const EXECUTION_TIMEOUT_NANOS: u64 = 7 * 24 * 60 * 60 * 1_000_000_000; // 7 days in nanoseconds

#[test]
fn test_claim_positive_flow() {
    let (pic, claim_canister, pool_canister, owner, ledger_id) = setup();

    let receiver_bytes = [3u8; 29];
    let receiver = Principal::from_slice(&receiver_bytes);
    let amount = Nat::from(1_000_000u64);
    let desc = String::from("Insurance payout for property damage");

    // Create claim client
    let mut claim_client = ClaimCanisterClient::new(&pic, claim_canister);
    let mut ledger_client = LedgerCanisterClient::new(&pic, ledger_id);

    // Transfer deposit to subaccount
    let deposit_amount = Nat::from(1_000_000u64);
    let subaccount = claim_client.connect(owner).get_claim_deposit_subaccount(
        owner,
        receiver,
        amount.clone(),
        pool_canister,
        desc.clone(),
    );
    transfer_to_deposit_subaccount(
        &mut ledger_client,
        owner,
        claim_canister,
        subaccount,
        deposit_amount,
    );

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

    // Verify claim is approved before execution timeout
    let claim_info = claim_client
        .connect(owner)
        .get_claim(claim_id)
        .expect("expected claim to exist after approval");
    assert_eq!(claim_info.status, ClaimStatus::Approved);

    // Advance time past execution timeout
    let execution_timeout = Duration::from_nanos(EXECUTION_TIMEOUT_NANOS);
    pic.advance_time(execution_timeout);

    // Get receiver's balance before execution
    let receiver_account = pool_canister::types::Account {
        owner: receiver,
        subaccount: None,
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
    let (pic, claim_canister, pool_canister, owner, ledger_id) = setup();

    let receiver_bytes = [3u8; 29];
    let receiver = Principal::from_slice(&receiver_bytes);
    let amount = Nat::from(1_000_000u64);
    let desc = String::from("Timelock test");

    let mut claim_client = ClaimCanisterClient::new(&pic, claim_canister);
    let mut ledger_client = LedgerCanisterClient::new(&pic, ledger_id);

    // Transfer deposit to subaccount
    let deposit_amount = Nat::from(1_000_000u64);
    let subaccount = claim_client.connect(owner).get_claim_deposit_subaccount(
        owner,
        receiver,
        amount.clone(),
        pool_canister,
        desc.clone(),
    );
    transfer_to_deposit_subaccount(
        &mut ledger_client,
        owner,
        claim_canister,
        subaccount,
        deposit_amount,
    );

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

    // Try to execute before execution timeout - should fail
    let exec_result = claim_client.connect(receiver).execute_claim(claim_id);
    assert_eq!(exec_result, Err(ClaimError::ExecutionTimeoutNotExpired));

    // Advance time past execution timeout
    let execution_timeout = Duration::from_nanos(EXECUTION_TIMEOUT_NANOS);
    pic.advance_time(execution_timeout);

    // Execute claim - should succeed
    claim_client
        .connect(receiver)
        .execute_claim(claim_id)
        .expect("execute_claim should succeed after execution timeout");

    // Verify claim status is Executed
    let final_info = claim_client
        .connect(owner)
        .get_claim(claim_id)
        .expect("expected claim to exist after execution");
    assert_eq!(final_info.status, ClaimStatus::Executed);
}

#[test]
fn test_cannot_execute_same_claim_multiple_times() {
    let (pic, claim_canister, pool_canister, owner, ledger_id) = setup();

    let receiver_bytes = [4u8; 29];
    let receiver = Principal::from_slice(&receiver_bytes);
    let amount = Nat::from(1_000_000u64);
    let desc = String::from("Double execute test");

    let mut claim_client = ClaimCanisterClient::new(&pic, claim_canister);
    let mut ledger_client = LedgerCanisterClient::new(&pic, ledger_id);

    // Transfer deposit to subaccount
    let deposit_amount = Nat::from(1_000_000u64);
    let subaccount = claim_client.connect(owner).get_claim_deposit_subaccount(
        owner,
        receiver,
        amount.clone(),
        pool_canister,
        desc.clone(),
    );
    transfer_to_deposit_subaccount(
        &mut ledger_client,
        owner,
        claim_canister,
        subaccount,
        deposit_amount,
    );

    // Create and approve claim
    let claim_id = claim_client
        .connect(owner)
        .add_claim(receiver, amount, pool_canister, desc)
        .expect("add_claim should succeed");

    claim_client
        .connect(owner)
        .approve_claim(claim_id)
        .expect("approve_claim should succeed");

    // Advance time past execution timeout
    let execution_timeout = Duration::from_nanos(EXECUTION_TIMEOUT_NANOS);
    pic.advance_time(execution_timeout);

    // First execution - should succeed
    let first_result = claim_client.connect(receiver).execute_claim(claim_id);
    assert_eq!(first_result, Ok(()));

    let claim_info1 = claim_client
        .connect(owner)
        .get_claim(claim_id)
        .expect("claim should exist");
    assert_eq!(claim_info1.status, ClaimStatus::Executed);

    // Second execution - should fail
    let second_result = claim_client.connect(receiver).execute_claim(claim_id);
    assert_eq!(second_result, Err(ClaimError::AlreadyExecuted));

    let claim_info2 = claim_client
        .connect(owner)
        .get_claim(claim_id)
        .expect("claim should still exist");
    assert_eq!(claim_info2.status, ClaimStatus::Executed);
}

#[test]
fn test_execute_before_approval_not_possible() {
    let (pic, claim_canister, pool_canister, owner, ledger_id) = setup();

    let receiver_bytes = [5u8; 29];
    let receiver = Principal::from_slice(&receiver_bytes);
    let amount = Nat::from(777u64);
    let desc = String::from("Execute before approval");

    let mut claim_client = ClaimCanisterClient::new(&pic, claim_canister);
    let mut ledger_client = LedgerCanisterClient::new(&pic, ledger_id);

    // Transfer deposit to subaccount
    let deposit_amount = Nat::from(1_000_000u64);
    let subaccount = claim_client.connect(owner).get_claim_deposit_subaccount(
        owner,
        receiver,
        amount.clone(),
        pool_canister,
        desc.clone(),
    );
    transfer_to_deposit_subaccount(
        &mut ledger_client,
        owner,
        claim_canister,
        subaccount,
        deposit_amount,
    );

    // Create claim but don't approve
    let claim_id = claim_client
        .connect(owner)
        .add_claim(receiver, amount, pool_canister, desc)
        .expect("add_claim should succeed");

    // Try to execute without approval - should fail
    let exec_result = claim_client.connect(receiver).execute_claim(claim_id);
    assert_eq!(exec_result, Err(ClaimError::NotApproved));
}

#[test]
fn test_only_owner_can_change_approvers() {
    let (pic, claim_canister, _pool_canister, owner, _ledger_id) = setup();

    let other_bytes = [9u8; 29];
    let other = Principal::from_slice(&other_bytes);

    let mut claim_client = ClaimCanisterClient::new(&pic, claim_canister);

    // Non-owner cannot add approver
    let add_result_non_owner = claim_client.connect(other).add_approver(other);
    assert_eq!(
        add_result_non_owner,
        Err(ClaimError::InsufficientPermissions)
    );

    // Owner can add approver
    let add_result_owner = claim_client.connect(owner).add_approver(other);
    assert_eq!(add_result_owner, Ok(()));

    // Non-owner cannot remove approver
    let remove_result_non_owner = claim_client.connect(other).remove_approver(other);
    assert_eq!(
        remove_result_non_owner,
        Err(ClaimError::InsufficientPermissions)
    );

    // Owner can remove approver
    let remove_result_owner = claim_client.connect(owner).remove_approver(other);
    assert_eq!(remove_result_owner, Ok(()));

    // Verify the approver was actually removed
    let is_approver = claim_client.connect(owner).is_approver(other);
    assert!(
        !is_approver,
        "Approver should be removed and no longer be able to approve"
    );
}

#[test]
fn test_claim_status_reverts_to_approved_on_slash_failure() {
    let (pic, claim_canister, pool_canister, owner, ledger_id) = setup();

    let receiver_bytes = [6u8; 29];
    let receiver = Principal::from_slice(&receiver_bytes);
    let amount = Nat::from(200_000_000_000u64);
    let desc = String::from("Slash failure test");

    let mut claim_client = ClaimCanisterClient::new(&pic, claim_canister);
    let mut ledger_client = LedgerCanisterClient::new(&pic, ledger_id);

    // Transfer deposit to subaccount
    let deposit_amount = Nat::from(1_000_000u64);
    let subaccount = claim_client.connect(owner).get_claim_deposit_subaccount(
        owner,
        receiver,
        amount.clone(),
        pool_canister,
        desc.clone(),
    );
    transfer_to_deposit_subaccount(
        &mut ledger_client,
        owner,
        claim_canister,
        subaccount,
        deposit_amount,
    );

    // Create and approve claim
    let claim_id = claim_client
        .connect(owner)
        .add_claim(receiver, amount, pool_canister, desc)
        .expect("add_claim should succeed");

    claim_client
        .connect(owner)
        .approve_claim(claim_id)
        .expect("approve_claim should succeed");

    // Advance time past execution timeout
    let execution_timeout = Duration::from_nanos(EXECUTION_TIMEOUT_NANOS);
    pic.advance_time(execution_timeout);

    // Execute claim (this will fail due to insufficient balance in pool)
    let exec_result = claim_client.connect(receiver).execute_claim(claim_id);
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

#[test]
fn test_claim_creation_requires_deposit() {
    let (pic, claim_canister, pool_canister, owner, ledger_id) = setup();

    let proposer_bytes = [10u8; 29];
    let proposer = Principal::from_slice(&proposer_bytes);
    let receiver_bytes = [11u8; 29];
    let receiver = Principal::from_slice(&receiver_bytes);

    let mut claim_client = ClaimCanisterClient::new(&pic, claim_canister);
    let mut ledger_client = LedgerCanisterClient::new(&pic, ledger_id);

    // Fund the proposer
    let proposer_account = pool_canister::types::Account {
        owner: proposer,
        subaccount: None,
    };
    let transfer_args = pool_canister::types::TransferArg {
        from_subaccount: None,
        to: proposer_account.clone(),
        amount: Nat::from(10_000_000u64),
        fee: Some(TRANSFER_FEE.clone()),
        memo: None,
        created_at_time: None,
    };
    ledger_client.connect(owner).icrc1_transfer(transfer_args);

    // Check the claim deposit requirement
    let required_deposit = claim_client.connect(proposer).get_claim_deposit();
    assert_eq!(required_deposit, Nat::from(1_000_000u64));

    // Get proposer balance before transferring to subaccount
    let balance_before = ledger_client
        .connect(proposer)
        .icrc1_balance_of(proposer_account.clone());

    // Define claim parameters
    let claim_amount = Nat::from(5_000_000u64);
    let claim_desc = "Test claim with deposit".to_string();

    // Transfer deposit to proposer's subaccount
    let subaccount = claim_client.connect(proposer).get_claim_deposit_subaccount(
        proposer,
        receiver,
        claim_amount.clone(),
        pool_canister,
        claim_desc.clone(),
    );
    transfer_to_deposit_subaccount(
        &mut ledger_client,
        proposer,
        claim_canister,
        subaccount,
        required_deposit.clone(),
    );

    // Create claim (deposit already in subaccount)
    let claim_id = claim_client
        .connect(proposer)
        .add_claim(receiver, claim_amount, pool_canister, claim_desc)
        .expect("add_claim should succeed");

    // Verify deposit was deducted from proposer's main account (when transferred to subaccount)
    let balance_after = ledger_client
        .connect(proposer)
        .icrc1_balance_of(proposer_account);
    let expected_balance = balance_before - required_deposit.clone() - TRANSFER_FEE.clone();
    assert_eq!(balance_after, expected_balance);

    // Verify claim has deposit amount stored
    let claim_info = claim_client.connect(proposer).get_claim(claim_id).unwrap();
    assert_eq!(claim_info.deposit_amount, required_deposit);
    assert_eq!(claim_info.proposer, proposer);
    assert_eq!(claim_info.status, ClaimStatus::Pending);
}

#[test]
fn test_deposit_not_returned_on_approval() {
    let (pic, claim_canister, pool_canister, owner, ledger_id) = setup();

    let proposer_bytes = [12u8; 29];
    let proposer = Principal::from_slice(&proposer_bytes);
    let receiver_bytes = [13u8; 29];
    let receiver = Principal::from_slice(&receiver_bytes);

    let mut claim_client = ClaimCanisterClient::new(&pic, claim_canister);
    let mut ledger_client = LedgerCanisterClient::new(&pic, ledger_id);

    // Fund the proposer
    let proposer_account = pool_canister::types::Account {
        owner: proposer,
        subaccount: None,
    };
    let transfer_args = pool_canister::types::TransferArg {
        from_subaccount: None,
        to: proposer_account.clone(),
        amount: Nat::from(10_000_000u64),
        fee: Some(TRANSFER_FEE.clone()),
        memo: None,
        created_at_time: None,
    };
    ledger_client.connect(owner).icrc1_transfer(transfer_args);

    // Define claim parameters
    let deposit_amount = Nat::from(1_000_000u64);
    let claim_amount = Nat::from(2_000_000u64);
    let claim_desc = "Approval test".to_string();

    // Transfer deposit to proposer's subaccount
    let subaccount = claim_client.connect(proposer).get_claim_deposit_subaccount(
        proposer,
        receiver,
        claim_amount.clone(),
        pool_canister,
        claim_desc.clone(),
    );
    transfer_to_deposit_subaccount(
        &mut ledger_client,
        proposer,
        claim_canister,
        subaccount,
        deposit_amount,
    );

    // Create claim
    let claim_id = claim_client
        .connect(proposer)
        .add_claim(receiver, claim_amount, pool_canister, claim_desc)
        .expect("add_claim should succeed");

    let balance_after_claim = ledger_client
        .connect(proposer)
        .icrc1_balance_of(proposer_account.clone());

    // Approve claim
    claim_client
        .connect(owner)
        .approve_claim(claim_id)
        .expect("approve_claim should succeed");

    // Verify deposit was NOT returned - balance should remain the same
    let balance_after_approval = ledger_client
        .connect(proposer)
        .icrc1_balance_of(proposer_account);
    assert_eq!(balance_after_approval, balance_after_claim);
}

#[test]
fn test_withdraw_deposit_from_pending_claim() {
    let (pic, claim_canister, pool_canister, owner, ledger_id) = setup();

    let proposer_bytes = [14u8; 29];
    let proposer = Principal::from_slice(&proposer_bytes);
    let receiver_bytes = [15u8; 29];
    let receiver = Principal::from_slice(&receiver_bytes);

    let mut claim_client = ClaimCanisterClient::new(&pic, claim_canister);
    let mut ledger_client = LedgerCanisterClient::new(&pic, ledger_id);

    // Fund the proposer
    let proposer_account = pool_canister::types::Account {
        owner: proposer,
        subaccount: None,
    };
    let transfer_args = pool_canister::types::TransferArg {
        from_subaccount: None,
        to: proposer_account.clone(),
        amount: Nat::from(10_000_000u64),
        fee: Some(TRANSFER_FEE.clone()),
        memo: None,
        created_at_time: None,
    };
    ledger_client.connect(owner).icrc1_transfer(transfer_args);

    // Define claim parameters
    let deposit_amount = Nat::from(1_000_000u64);
    let claim_amount = Nat::from(3_000_000u64);
    let claim_desc = "Withdraw deposit test".to_string();

    // Transfer deposit to proposer's subaccount
    let subaccount = claim_client.connect(proposer).get_claim_deposit_subaccount(
        proposer,
        receiver,
        claim_amount.clone(),
        pool_canister,
        claim_desc.clone(),
    );
    transfer_to_deposit_subaccount(
        &mut ledger_client,
        proposer,
        claim_canister,
        subaccount,
        deposit_amount,
    );

    // Create claim
    let claim_id = claim_client
        .connect(proposer)
        .add_claim(receiver, claim_amount, pool_canister, claim_desc)
        .expect("add_claim should succeed");

    let balance_after_claim = ledger_client
        .connect(proposer)
        .icrc1_balance_of(proposer_account.clone());

    // Advance time past approval period (7 days)
    let approval_period = Duration::from_nanos(APPROVAL_PERIOD_NANOS);
    pic.advance_time(approval_period);

    // Withdraw should succeed after approval period expires
    claim_client
        .connect(proposer)
        .withdraw_deposit(claim_id)
        .expect("withdraw_deposit should succeed");

    // Verify deposit was returned
    let balance_after_withdraw = ledger_client
        .connect(proposer)
        .icrc1_balance_of(proposer_account);
    let expected_balance = balance_after_claim + Nat::from(1_000_000u64) - TRANSFER_FEE.clone();
    assert_eq!(balance_after_withdraw, expected_balance);
}

#[test]
fn test_mark_as_spam_blocks_deposit_withdrawal() {
    let (pic, claim_canister, pool_canister, owner, ledger_id) = setup();

    let proposer_bytes = [16u8; 29];
    let proposer = Principal::from_slice(&proposer_bytes);
    let receiver_bytes = [17u8; 29];
    let receiver = Principal::from_slice(&receiver_bytes);

    let mut claim_client = ClaimCanisterClient::new(&pic, claim_canister);
    let mut ledger_client = LedgerCanisterClient::new(&pic, ledger_id);

    // Fund the proposer
    let proposer_account = pool_canister::types::Account {
        owner: proposer,
        subaccount: None,
    };
    let transfer_args = pool_canister::types::TransferArg {
        from_subaccount: None,
        to: proposer_account.clone(),
        amount: Nat::from(10_000_000u64),
        fee: Some(TRANSFER_FEE.clone()),
        memo: None,
        created_at_time: None,
    };
    ledger_client.connect(owner).icrc1_transfer(transfer_args);

    // Define claim parameters
    let deposit_amount = Nat::from(1_000_000u64);
    let claim_amount = Nat::from(2_000_000u64);
    let claim_desc = "Spam test".to_string();

    // Transfer deposit to proposer's subaccount
    let subaccount = claim_client.connect(proposer).get_claim_deposit_subaccount(
        proposer,
        receiver,
        claim_amount.clone(),
        pool_canister,
        claim_desc.clone(),
    );
    transfer_to_deposit_subaccount(
        &mut ledger_client,
        proposer,
        claim_canister,
        subaccount,
        deposit_amount,
    );

    // Create claim
    let claim_id = claim_client
        .connect(proposer)
        .add_claim(receiver, claim_amount, pool_canister, claim_desc)
        .expect("add_claim should succeed");

    let balance_after_claim = ledger_client
        .connect(proposer)
        .icrc1_balance_of(proposer_account.clone());

    // Mark as spam
    claim_client
        .connect(owner)
        .mark_as_spam(claim_id)
        .expect("mark_as_spam should succeed");

    // Verify claim is marked as spam
    let claim_info = claim_client.connect(owner).get_claim(claim_id).unwrap();
    assert_eq!(claim_info.status, ClaimStatus::Spam);

    // Withdraw deposit should FAIL for spam claims
    let withdraw_result = claim_client.connect(proposer).withdraw_deposit(claim_id);
    assert_eq!(
        withdraw_result,
        Err(ClaimError::AlreadyMarkedAsSpam),
        "Withdrawal should be blocked for spam claims"
    );

    // Verify balance hasn't changed (deposit was not returned)
    let balance_after_withdraw_attempt = ledger_client
        .connect(proposer)
        .icrc1_balance_of(proposer_account);
    assert_eq!(
        balance_after_withdraw_attempt, balance_after_claim,
        "Balance should remain unchanged"
    );
}

#[test]
fn test_can_mark_approved_claim_as_spam() {
    let (pic, claim_canister, pool_canister, owner, ledger_id) = setup();

    let proposer_bytes = [18u8; 29];
    let proposer = Principal::from_slice(&proposer_bytes);
    let receiver_bytes = [19u8; 29];
    let receiver = Principal::from_slice(&receiver_bytes);

    let mut claim_client = ClaimCanisterClient::new(&pic, claim_canister);
    let mut ledger_client = LedgerCanisterClient::new(&pic, ledger_id);

    // Fund the proposer
    let proposer_account = pool_canister::types::Account {
        owner: proposer,
        subaccount: None,
    };
    let transfer_args = pool_canister::types::TransferArg {
        from_subaccount: None,
        to: proposer_account,
        amount: Nat::from(10_000_000u64),
        fee: Some(TRANSFER_FEE.clone()),
        memo: None,
        created_at_time: None,
    };
    ledger_client.connect(owner).icrc1_transfer(transfer_args);

    // Define claim parameters
    let deposit_amount = Nat::from(1_000_000u64);
    let claim_amount = Nat::from(1_000_000u64);
    let claim_desc = "Can mark approved as spam".to_string();

    // Transfer deposit to proposer's subaccount
    let subaccount = claim_client.connect(proposer).get_claim_deposit_subaccount(
        proposer,
        receiver,
        claim_amount.clone(),
        pool_canister,
        claim_desc.clone(),
    );
    transfer_to_deposit_subaccount(
        &mut ledger_client,
        proposer,
        claim_canister,
        subaccount,
        deposit_amount,
    );

    // Create and approve claim
    let claim_id = claim_client
        .connect(proposer)
        .add_claim(receiver, claim_amount, pool_canister, claim_desc)
        .expect("add_claim should succeed");

    claim_client
        .connect(owner)
        .approve_claim(claim_id)
        .expect("approve_claim should succeed");

    // Mark approved claim as spam - should succeed now
    claim_client
        .connect(owner)
        .mark_as_spam(claim_id)
        .expect("mark_as_spam should succeed for approved claims");

    // Verify claim is marked as spam
    let claim_info = claim_client.connect(owner).get_claim(claim_id).unwrap();
    assert_eq!(claim_info.status, ClaimStatus::Spam);
}

#[test]
fn test_only_approver_can_mark_as_spam() {
    let (pic, claim_canister, pool_canister, owner, ledger_id) = setup();

    let proposer_bytes = [20u8; 29];
    let proposer = Principal::from_slice(&proposer_bytes);
    let receiver_bytes = [21u8; 29];
    let receiver = Principal::from_slice(&receiver_bytes);
    let non_approver_bytes = [22u8; 29];
    let non_approver = Principal::from_slice(&non_approver_bytes);

    let mut claim_client = ClaimCanisterClient::new(&pic, claim_canister);
    let mut ledger_client = LedgerCanisterClient::new(&pic, ledger_id);

    // Fund the proposer
    let proposer_account = pool_canister::types::Account {
        owner: proposer,
        subaccount: None,
    };
    let transfer_args = pool_canister::types::TransferArg {
        from_subaccount: None,
        to: proposer_account,
        amount: Nat::from(10_000_000u64),
        fee: Some(TRANSFER_FEE.clone()),
        memo: None,
        created_at_time: None,
    };
    ledger_client.connect(owner).icrc1_transfer(transfer_args);

    // Define claim parameters
    let deposit_amount = Nat::from(1_000_000u64);
    let claim_amount = Nat::from(1_000_000u64);
    let claim_desc = "Only approver can mark spam".to_string();

    // Transfer deposit to proposer's subaccount
    let subaccount = claim_client.connect(proposer).get_claim_deposit_subaccount(
        proposer,
        receiver,
        claim_amount.clone(),
        pool_canister,
        claim_desc.clone(),
    );
    transfer_to_deposit_subaccount(
        &mut ledger_client,
        proposer,
        claim_canister,
        subaccount,
        deposit_amount,
    );

    // Create claim
    let claim_id = claim_client
        .connect(proposer)
        .add_claim(receiver, claim_amount, pool_canister, claim_desc)
        .expect("add_claim should succeed");

    // Non-approver tries to mark as spam - should fail
    let mark_result = claim_client.connect(non_approver).mark_as_spam(claim_id);
    assert_eq!(mark_result, Err(ClaimError::NotApprover));

    // Approver (owner) marks as spam - should succeed
    claim_client
        .connect(owner)
        .mark_as_spam(claim_id)
        .expect("mark_as_spam should succeed for approver");

    let claim_info = claim_client.connect(owner).get_claim(claim_id).unwrap();
    assert_eq!(claim_info.status, ClaimStatus::Spam);
}

#[test]
fn test_only_owner_can_set_claim_deposit() {
    let (pic, claim_canister, _pool_canister, owner, _ledger_id) = setup();

    let non_owner_bytes = [23u8; 29];
    let non_owner = Principal::from_slice(&non_owner_bytes);

    let mut claim_client = ClaimCanisterClient::new(&pic, claim_canister);

    // Non-owner tries to set claim deposit - should fail
    let set_result = claim_client
        .connect(non_owner)
        .set_claim_deposit(Nat::from(2_000_000u64));
    assert_eq!(set_result, Err(ClaimError::InsufficientPermissions));

    // Owner sets claim deposit - should succeed
    claim_client
        .connect(owner)
        .set_claim_deposit(Nat::from(2_000_000u64))
        .expect("set_claim_deposit should succeed for owner");

    // Verify the deposit was updated
    let new_deposit = claim_client.connect(owner).get_claim_deposit();
    assert_eq!(new_deposit, Nat::from(2_000_000u64));
}

#[test]
fn test_only_proposer_can_withdraw_deposit() {
    let (pic, claim_canister, pool_canister, owner, ledger_id) = setup();

    let proposer_bytes = [24u8; 29];
    let proposer = Principal::from_slice(&proposer_bytes);
    let receiver_bytes = [25u8; 29];
    let receiver = Principal::from_slice(&receiver_bytes);
    let other_bytes = [26u8; 29];
    let other = Principal::from_slice(&other_bytes);

    let mut claim_client = ClaimCanisterClient::new(&pic, claim_canister);
    let mut ledger_client = LedgerCanisterClient::new(&pic, ledger_id);

    // Fund the proposer
    let proposer_account = pool_canister::types::Account {
        owner: proposer,
        subaccount: None,
    };
    let transfer_args = pool_canister::types::TransferArg {
        from_subaccount: None,
        to: proposer_account,
        amount: Nat::from(10_000_000u64),
        fee: Some(TRANSFER_FEE.clone()),
        memo: None,
        created_at_time: None,
    };
    ledger_client.connect(owner).icrc1_transfer(transfer_args);

    // Define claim parameters
    let deposit_amount = Nat::from(1_000_000u64);
    let claim_amount = Nat::from(1_000_000u64);
    let claim_desc = "Only proposer can withdraw".to_string();

    // Transfer deposit to proposer's subaccount
    let subaccount = claim_client.connect(proposer).get_claim_deposit_subaccount(
        proposer,
        receiver,
        claim_amount.clone(),
        pool_canister,
        claim_desc.clone(),
    );
    transfer_to_deposit_subaccount(
        &mut ledger_client,
        proposer,
        claim_canister,
        subaccount,
        deposit_amount,
    );

    // Create claim
    let claim_id = claim_client
        .connect(proposer)
        .add_claim(receiver, claim_amount, pool_canister, claim_desc)
        .expect("add_claim should succeed");

    // Other user tries to withdraw - should fail
    let withdraw_result = claim_client.connect(other).withdraw_deposit(claim_id);
    assert_eq!(withdraw_result, Err(ClaimError::NotProposer));

    // Advance time past approval period (7 days)
    let approval_period = Duration::from_nanos(APPROVAL_PERIOD_NANOS);
    pic.advance_time(approval_period);

    // Proposer withdraws - should succeed after approval period expires

    let claim_info = claim_client.connect(proposer).get_claim(claim_id).unwrap();
    println!("claim_info after approval period: {:?}", claim_info);
    claim_client
        .connect(proposer)
        .withdraw_deposit(claim_id)
        .expect("withdraw_deposit should succeed for proposer");
}

#[test]
fn test_can_withdraw_deposit_from_approved_claim() {
    let (pic, claim_canister, pool_canister, owner, ledger_id) = setup();

    let proposer_bytes = [27u8; 29];
    let proposer = Principal::from_slice(&proposer_bytes);
    let receiver_bytes = [28u8; 29];
    let receiver = Principal::from_slice(&receiver_bytes);

    let mut claim_client = ClaimCanisterClient::new(&pic, claim_canister);
    let mut ledger_client = LedgerCanisterClient::new(&pic, ledger_id);

    // Fund the proposer
    let proposer_account = pool_canister::types::Account {
        owner: proposer,
        subaccount: None,
    };
    let transfer_args = pool_canister::types::TransferArg {
        from_subaccount: None,
        to: proposer_account.clone(),
        amount: Nat::from(10_000_000u64),
        fee: Some(TRANSFER_FEE.clone()),
        memo: None,
        created_at_time: None,
    };
    ledger_client.connect(owner).icrc1_transfer(transfer_args);

    // Define claim parameters
    let deposit_amount = Nat::from(1_000_000u64);
    let claim_amount = Nat::from(1_000_000u64);
    let claim_desc = "Can withdraw from approved".to_string();

    // Transfer deposit to proposer's subaccount
    let subaccount = claim_client.connect(proposer).get_claim_deposit_subaccount(
        proposer,
        receiver,
        claim_amount.clone(),
        pool_canister,
        claim_desc.clone(),
    );
    transfer_to_deposit_subaccount(
        &mut ledger_client,
        proposer,
        claim_canister,
        subaccount,
        deposit_amount,
    );

    // Create claim
    let claim_id = claim_client
        .connect(proposer)
        .add_claim(receiver, claim_amount, pool_canister, claim_desc)
        .expect("add_claim should succeed");

    let balance_after_claim = ledger_client
        .connect(proposer)
        .icrc1_balance_of(proposer_account.clone());

    // Approve claim
    claim_client
        .connect(owner)
        .approve_claim(claim_id)
        .expect("approve_claim should succeed");

    // Try to withdraw before approval period expires - should fail
    let withdraw_result = claim_client.connect(proposer).withdraw_deposit(claim_id);
    assert_eq!(
        withdraw_result,
        Err(ClaimError::AlreadyApproved),
        "Withdraw should fail before approval period expires"
    );

    // Advance time past approval period (7 days)
    let approval_period = Duration::from_nanos(EXECUTION_TIMEOUT_NANOS);
    pic.advance_time(approval_period);

    // Withdraw deposit should succeed after approval period has expired
    claim_client
        .connect(proposer)
        .withdraw_deposit(claim_id)
        .expect("withdraw_deposit should succeed after approval period expires");

    // Verify deposit was returned
    let balance_after_withdraw = ledger_client
        .connect(proposer)
        .icrc1_balance_of(proposer_account);
    let expected_balance = balance_after_claim + Nat::from(1_000_000u64) - TRANSFER_FEE.clone();
    assert_eq!(balance_after_withdraw, expected_balance);

    // Verify claim's deposit amount is now 0
    let claim_info = claim_client.connect(proposer).get_claim(claim_id).unwrap();
    assert_eq!(
        claim_info.deposit_amount,
        Nat::from(0u64),
        "Deposit amount should be 0 after withdrawal"
    );

    // Second call to withdraw_deposit should fail with NoDepositToWithdraw
    let second_withdraw_result = claim_client.connect(proposer).withdraw_deposit(claim_id);
    assert_eq!(
        second_withdraw_result,
        Err(ClaimError::NoDepositToWithdraw),
        "Second withdrawal should fail"
    );
}
