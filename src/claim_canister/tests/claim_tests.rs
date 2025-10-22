mod setup;
mod utils;
use candid::{Nat, Principal};
use claim_canister::types::{ClaimError, ClaimStatus};
use claim_canister::TIMELOCK_DURATION;
use commons::{ClaimCanisterClient, LedgerCanisterClient};
use pool_canister::TRANSFER_FEE;
use setup::setup;
use std::time::Duration;
use utils::approve_deposit;

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

    // Approve the claim canister to spend deposit on behalf of owner
    let deposit_amount = Nat::from(1_000_000u64);
    approve_deposit(&mut ledger_client, owner, claim_canister, deposit_amount);

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

    // Approve the claim canister to spend deposit on behalf of owner
    let deposit_amount = Nat::from(1_000_000u64);
    approve_deposit(&mut ledger_client, owner, claim_canister, deposit_amount);

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
    let exec_result = claim_client.connect(receiver).execute_claim(claim_id);
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
    let (pic, claim_canister, pool_canister, owner, ledger_id) = setup();

    let receiver_bytes = [4u8; 29];
    let receiver = Principal::from_slice(&receiver_bytes);
    let amount = Nat::from(1_000_000u64);
    let desc = String::from("Double execute test");

    let mut claim_client = ClaimCanisterClient::new(&pic, claim_canister);
    let mut ledger_client = LedgerCanisterClient::new(&pic, ledger_id);

    // Approve the claim canister to spend deposit on behalf of owner
    let deposit_amount = Nat::from(1_000_000u64);
    approve_deposit(&mut ledger_client, owner, claim_canister, deposit_amount);

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

    // Approve the claim canister to spend deposit on behalf of owner
    let deposit_amount = Nat::from(1_000_000u64);
    approve_deposit(&mut ledger_client, owner, claim_canister, deposit_amount);

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

    // Approve the claim canister to spend deposit on behalf of owner
    let deposit_amount = Nat::from(1_000_000u64);
    approve_deposit(&mut ledger_client, owner, claim_canister, deposit_amount);

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
    ledger_client
        .connect(owner)
        .icrc1_transfer(transfer_args);

    // Check the claim deposit requirement
    let required_deposit = claim_client.connect(proposer).get_claim_deposit();
    assert_eq!(required_deposit, Nat::from(1_000_000u64));

    // Approve the claim canister to spend tokens on behalf of proposer
    approve_deposit(&mut ledger_client, proposer, claim_canister, required_deposit.clone());

    // Get proposer balance before claim creation (after approval)
    let balance_before = ledger_client.connect(proposer).icrc1_balance_of(proposer_account.clone());

    // Create claim (should deduct deposit)
    let claim_id = claim_client
        .connect(proposer)
        .add_claim(
            receiver,
            Nat::from(5_000_000u64),
            pool_canister,
            "Test claim with deposit".to_string(),
        )
        .expect("add_claim should succeed");

    // Verify deposit was deducted from proposer
    let balance_after = ledger_client.connect(proposer).icrc1_balance_of(proposer_account);
    let expected_balance = balance_before - required_deposit.clone() - TRANSFER_FEE.clone();
    assert_eq!(balance_after, expected_balance);

    // Verify claim has deposit amount stored
    let claim_info = claim_client.connect(proposer).get_claim(claim_id).unwrap();
    assert_eq!(claim_info.deposit_amount, required_deposit);
    assert_eq!(claim_info.proposer, proposer);
    assert!(!claim_info.spam);
}

#[test]
fn test_deposit_returned_on_approval() {
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
    ledger_client
        .connect(owner)
        .icrc1_transfer(transfer_args);

    // Approve the claim canister to spend deposit on behalf of proposer
    let deposit_amount = Nat::from(1_000_000u64);
    approve_deposit(&mut ledger_client, proposer, claim_canister, deposit_amount);

    // Create claim
    let claim_id = claim_client
        .connect(proposer)
        .add_claim(
            receiver,
            Nat::from(2_000_000u64),
            pool_canister,
            "Approval test".to_string(),
        )
        .expect("add_claim should succeed");

    let balance_after_claim = ledger_client.connect(proposer).icrc1_balance_of(proposer_account.clone());

    // Approve claim (should return deposit)
    claim_client
        .connect(owner)
        .approve_claim(claim_id)
        .expect("approve_claim should succeed");

    // Verify deposit was returned
    let balance_after_approval = ledger_client.connect(proposer).icrc1_balance_of(proposer_account);

    // After approval, proposer should have received deposit back minus one transfer fee for the refund
    let expected_balance = balance_after_claim.clone() + Nat::from(1_000_000u64) - TRANSFER_FEE.clone();
    assert_eq!(balance_after_approval, expected_balance);
}

#[test]
fn test_withdraw_deposit_after_approval_period() {
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
    ledger_client
        .connect(owner)
        .icrc1_transfer(transfer_args);

    // Approve the claim canister to spend deposit on behalf of proposer
    let deposit_amount = Nat::from(1_000_000u64);
    approve_deposit(&mut ledger_client, proposer, claim_canister, deposit_amount);

    // Create claim
    let claim_id = claim_client
        .connect(proposer)
        .add_claim(
            receiver,
            Nat::from(3_000_000u64),
            pool_canister,
            "Withdraw deposit test".to_string(),
        )
        .expect("add_claim should succeed");

    let balance_after_claim = ledger_client.connect(proposer).icrc1_balance_of(proposer_account.clone());

    // Try to withdraw before approval period expires - should fail
    let withdraw_result_early = claim_client.connect(proposer).withdraw_deposit(claim_id);
    assert_eq!(withdraw_result_early, Err(ClaimError::ApprovalPeriodNotExpired));

    // Advance time past approval period (7 days)
    let approval_period = Duration::from_nanos(7 * 24 * 60 * 60 * 1_000_000_000u64);
    pic.advance_time(approval_period);

    // Now withdraw should succeed
    claim_client
        .connect(proposer)
        .withdraw_deposit(claim_id)
        .expect("withdraw_deposit should succeed after approval period");

    // Verify deposit was returned
    let balance_after_withdraw = ledger_client.connect(proposer).icrc1_balance_of(proposer_account);
    let expected_balance = balance_after_claim + Nat::from(1_000_000u64) - TRANSFER_FEE.clone();
    assert_eq!(balance_after_withdraw, expected_balance);
}

#[test]
fn test_mark_as_spam_prevents_deposit_withdrawal() {
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
    ledger_client
        .connect(owner)
        .icrc1_transfer(transfer_args);

    // Approve the claim canister to spend deposit on behalf of proposer
    let deposit_amount = Nat::from(1_000_000u64);
    approve_deposit(&mut ledger_client, proposer, claim_canister, deposit_amount);

    // Create claim
    let claim_id = claim_client
        .connect(proposer)
        .add_claim(
            receiver,
            Nat::from(2_000_000u64),
            pool_canister,
            "Spam test".to_string(),
        )
        .expect("add_claim should succeed");

    // Mark as spam
    claim_client
        .connect(owner)
        .mark_as_spam(claim_id)
        .expect("mark_as_spam should succeed");

    // Verify claim is marked as spam
    let claim_info = claim_client.connect(owner).get_claim(claim_id).unwrap();
    assert!(claim_info.spam);

    // Advance time past approval period
    let approval_period = Duration::from_nanos(7 * 24 * 60 * 60 * 1_000_000_000u64);
    pic.advance_time(approval_period);

    // Try to withdraw deposit - should fail because claim is spam
    let withdraw_result = claim_client.connect(proposer).withdraw_deposit(claim_id);
    assert_eq!(withdraw_result, Err(ClaimError::CannotWithdrawSpamClaim));
}

#[test]
fn test_cannot_mark_approved_claim_as_spam() {
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
    ledger_client
        .connect(owner)
        .icrc1_transfer(transfer_args);

    // Approve the claim canister to spend deposit on behalf of proposer
    let deposit_amount = Nat::from(1_000_000u64);
    approve_deposit(&mut ledger_client, proposer, claim_canister, deposit_amount);

    // Create and approve claim
    let claim_id = claim_client
        .connect(proposer)
        .add_claim(
            receiver,
            Nat::from(1_000_000u64),
            pool_canister,
            "Cannot mark approved as spam".to_string(),
        )
        .expect("add_claim should succeed");

    claim_client
        .connect(owner)
        .approve_claim(claim_id)
        .expect("approve_claim should succeed");

    // Try to mark approved claim as spam - should fail
    let mark_result = claim_client.connect(owner).mark_as_spam(claim_id);
    assert_eq!(mark_result, Err(ClaimError::AlreadyApproved));
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
    ledger_client
        .connect(owner)
        .icrc1_transfer(transfer_args);

    // Approve the claim canister to spend deposit on behalf of proposer
    let deposit_amount = Nat::from(1_000_000u64);
    approve_deposit(&mut ledger_client, proposer, claim_canister, deposit_amount);

    // Create claim
    let claim_id = claim_client
        .connect(proposer)
        .add_claim(
            receiver,
            Nat::from(1_000_000u64),
            pool_canister,
            "Only approver can mark spam".to_string(),
        )
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
    assert!(claim_info.spam);
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
    ledger_client
        .connect(owner)
        .icrc1_transfer(transfer_args);

    // Approve the claim canister to spend deposit on behalf of proposer
    let deposit_amount = Nat::from(1_000_000u64);
    approve_deposit(&mut ledger_client, proposer, claim_canister, deposit_amount);

    // Create claim
    let claim_id = claim_client
        .connect(proposer)
        .add_claim(
            receiver,
            Nat::from(1_000_000u64),
            pool_canister,
            "Only proposer can withdraw".to_string(),
        )
        .expect("add_claim should succeed");

    // Advance time past approval period
    let approval_period = Duration::from_nanos(7 * 24 * 60 * 60 * 1_000_000_000u64);
    pic.advance_time(approval_period);

    // Other user tries to withdraw - should fail
    let withdraw_result = claim_client.connect(other).withdraw_deposit(claim_id);
    assert_eq!(withdraw_result, Err(ClaimError::NotProposer));

    // Proposer withdraws - should succeed
    claim_client
        .connect(proposer)
        .withdraw_deposit(claim_id)
        .expect("withdraw_deposit should succeed for proposer");
}

#[test]
fn test_cannot_withdraw_deposit_from_approved_claim() {
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
        to: proposer_account,
        amount: Nat::from(10_000_000u64),
        fee: Some(TRANSFER_FEE.clone()),
        memo: None,
        created_at_time: None,
    };
    ledger_client
        .connect(owner)
        .icrc1_transfer(transfer_args);

    // Approve the claim canister to spend deposit on behalf of proposer
    let deposit_amount = Nat::from(1_000_000u64);
    approve_deposit(&mut ledger_client, proposer, claim_canister, deposit_amount);

    // Create claim
    let claim_id = claim_client
        .connect(proposer)
        .add_claim(
            receiver,
            Nat::from(1_000_000u64),
            pool_canister,
            "Cannot withdraw from approved".to_string(),
        )
        .expect("add_claim should succeed");

    // Approve claim
    claim_client
        .connect(owner)
        .approve_claim(claim_id)
        .expect("approve_claim should succeed");

    // Try to withdraw deposit - should fail
    let withdraw_result = claim_client.connect(proposer).withdraw_deposit(claim_id);
    assert_eq!(withdraw_result, Err(ClaimError::CannotWithdrawApprovedClaim));
}
