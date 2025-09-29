mod setup;
use candid::{decode_one, encode_args, Nat, Principal};
use claim_canister::types::{ClaimError, ClaimInfo, ClaimStatus};
use claim_canister::TIMELOCK_DURATION;
use setup::setup;
use std::time::Duration;

#[test]
fn test_claim_positive_flow() {
    let (pic, claim_canister, pool_canister, owner) = setup();

    let receiver_bytes = [3u8; 29];
    let receiver = Principal::from_slice(&receiver_bytes);
    let amount = Nat::from(1000u64);
    let desc = String::from("Insurance payout for property damage");

    let res = pic
        .update_call(
            claim_canister,
            owner,
            "add_claim",
            encode_args((receiver, amount.clone(), pool_canister, desc)).unwrap(),
        )
        .expect("Failed to add claim");

    let decoded: Result<Result<u64, ClaimError>, _> = decode_one(&res);
    let claim_id = decoded
        .expect("Failed to decode claim_id")
        .expect("add_claim returned Err");

    let approve_res = pic
        .update_call(
            claim_canister,
            owner,
            "approve_claim",
            encode_args((claim_id,)).unwrap(),
        )
        .expect("Approval transport failed");
    let approve_result: Result<(), ClaimError> = decode_one(&approve_res).unwrap();
    assert_eq!(approve_result, Ok(()));

    // Verify claim is approved before timelock
    let claim_info_res = pic
        .query_call(
            claim_canister,
            owner,
            "get_claim",
            encode_args((claim_id,)).unwrap(),
        )
        .expect("get_claim transport failed");
    let claim_info: Option<ClaimInfo> =
        decode_one(&claim_info_res).expect("decode ClaimInfo failed");
    assert!(
        claim_info.is_some(),
        "expected claim to exist after approval"
    );
    let claim_info = claim_info.unwrap();
    assert_eq!(claim_info.status, ClaimStatus::Approved);

    let one_day_plus_grace: Duration = Duration::from_nanos(TIMELOCK_DURATION);
    pic.advance_time(one_day_plus_grace);

    // execute_claim
    let exec_res = pic
        .update_call(
            claim_canister,
            receiver,
            "execute_claim",
            encode_args((claim_id,)).unwrap(),
        )
        .expect("execute_claim transport failed");
    let _exec_result: Result<(), ClaimError> = decode_one(&exec_res).unwrap();
 

    // Verify claim remains approved
    let final_claim_res = pic
        .query_call(
            claim_canister,
            owner,
            "get_claim",
            encode_args((claim_id,)).unwrap(),
        )
        .expect("final get_claim transport failed");
    let final_info: Option<ClaimInfo> =
        decode_one(&final_claim_res).expect("decode final ClaimInfo failed");
    assert!(
        final_info.is_some(),
        "expected claim to exist after failed execution"
    );
    let final_info = final_info.unwrap();
    assert_eq!(final_info.status, ClaimStatus::Approved);
}

#[test]
fn test_execute_only_after_timelock() {
    let (pic, claim_canister, pool_canister, owner) = setup();

    let receiver_bytes = [3u8; 29];
    let receiver = Principal::from_slice(&receiver_bytes);
    let amount = Nat::from(1_000_000u64);
    let desc = String::from("Timelock test");

    let res = pic
        .update_call(
            claim_canister,
            owner,
            "add_claim",
            encode_args((receiver, amount.clone(), pool_canister, desc)).unwrap(),
        )
        .expect("add_claim transport failed");
    let claim_id: Result<Result<u64, ClaimError>, _> = decode_one(&res);
    let claim_id = claim_id
        .expect("decode claim_id failed")
        .expect("add_claim returned Err");

    let approve_res = pic
        .update_call(
            claim_canister,
            owner,
            "approve_claim",
            encode_args((claim_id,)).unwrap(),
        )
        .expect("approve transport failed");
    let approve_result: Result<(), ClaimError> = decode_one(&approve_res).unwrap();
    assert_eq!(approve_result, Ok(()));

    // Before timelock
    let exec_res = pic
        .update_call(
            claim_canister,
            receiver,
            "execute_claim",
            encode_args((claim_id,)).unwrap(),
        )
        .expect("execute transport failed");
    let exec_result: Result<(), ClaimError> = decode_one(&exec_res).unwrap();
    assert_eq!(exec_result, Err(ClaimError::TimelockNotExpired));

    // After timelock
    let one_day_plus_grace: Duration = Duration::from_nanos(TIMELOCK_DURATION);
    pic.advance_time(one_day_plus_grace);
    let exec_res2 = pic
        .update_call(
            claim_canister,
            receiver,
            "execute_claim",
            encode_args((claim_id,)).unwrap(),
        )
        .expect("execute transport failed");
    let exec_result2: Result<(), ClaimError> = decode_one(&exec_res2).unwrap();
    assert_eq!(exec_result2, Ok(()));
}

#[test]
fn test_cannot_execute_same_claim_multiple_times() {
    let (pic, claim_canister, pool_canister, owner) = setup();

    let receiver_bytes = [4u8; 29];
    let receiver = Principal::from_slice(&receiver_bytes);
    let amount = Nat::from(1_000_000u64);
    let desc = String::from("Double execute test");

    // Create claim
    let res = pic
        .update_call(
            claim_canister,
            owner,
            "add_claim",
            encode_args((receiver, amount, pool_canister, desc)).unwrap(),
        )
        .unwrap();
    let claim_id: u64 = decode_one::<Result<u64, ClaimError>>(&res)
        .unwrap()
        .unwrap();

    // Approve claim
    let approve_res = pic
        .update_call(
            claim_canister,
            owner,
            "approve_claim",
            encode_args((claim_id,)).unwrap(),
        )
        .unwrap();
    let approve_result: Result<(), ClaimError> = decode_one(&approve_res).unwrap();
    assert_eq!(approve_result, Ok(()));

    pic.advance_time(Duration::from_nanos(TIMELOCK_DURATION));

    // First execution attempt
    let exec_res1 = pic
        .update_call(
            claim_canister,
            receiver,
            "execute_claim",
            encode_args((claim_id,)).unwrap(),
        )
        .unwrap();
    let first_result: Result<(), ClaimError> = decode_one(&exec_res1).unwrap();

    // Check first execution result and claim status
    let claim_info_res1 = pic
        .query_call(
            claim_canister,
            owner,
            "get_claim",
            encode_args((claim_id,)).unwrap(),
        )
        .unwrap();
    let claim_info1: Option<ClaimInfo> = decode_one(&claim_info_res1).unwrap();
    let claim_info1 = claim_info1.unwrap();

    // Second execution attempt
    let exec_res2 = pic
        .update_call(
            claim_canister,
            receiver,
            "execute_claim",
            encode_args((claim_id,)).unwrap(),
        )
        .unwrap();
    let second_result: Result<(), ClaimError> = decode_one(&exec_res2).unwrap();

    // Check second execution result and claim status
    let claim_info_res2 = pic
        .query_call(
            claim_canister,
            owner,
            "get_claim",
            encode_args((claim_id,)).unwrap(),
        )
        .unwrap();
    let claim_info2: Option<ClaimInfo> = decode_one(&claim_info_res2).unwrap();
    let claim_info2 = claim_info2.unwrap();

    // the first execution should succeed
    assert_eq!(first_result, Ok(()));
    assert_eq!(claim_info1.status, ClaimStatus::Executed);

    // Second execution should fail because claim is already executed
    assert_eq!(second_result, Err(ClaimError::AlreadyExecuted));
    assert_eq!(claim_info2.status, ClaimStatus::Executed);
}

#[test]
fn test_execute_before_approval_not_possible() {
    let (pic, claim_canister, pool_canister, owner) = setup();

    let receiver_bytes = [5u8; 29];
    let receiver = Principal::from_slice(&receiver_bytes);
    let amount = Nat::from(777u64);
    let desc = String::from("Execute before approval");

    let res = pic
        .update_call(
            claim_canister,
            owner,
            "add_claim",
            encode_args((receiver, amount.clone(), pool_canister, desc)).unwrap(),
        )
        .expect("add_claim transport failed");
    let claim_id: Result<Result<u64, ClaimError>, _> = decode_one(&res);
    let claim_id = claim_id
        .expect("decode claim_id failed")
        .expect("add_claim returned Err");

    // Try to execute without approval
    let exec_res = pic
        .update_call(
            claim_canister,
            receiver,
            "execute_claim",
            encode_args((claim_id,)).unwrap(),
        )
        .expect("execute transport failed");
    let exec_result: Result<(), ClaimError> = decode_one(&exec_res).unwrap();
    assert_eq!(exec_result, Err(ClaimError::NotApproved));
}

#[test]
fn test_only_owner_can_change_approvers() {
    let (pic, claim_canister, _pool_canister, owner) = setup();

    let other_bytes = [9u8; 29];
    let other = Principal::from_slice(&other_bytes);

    // Non-owner cannot add approver
    let add_res_non_owner = pic
        .update_call(
            claim_canister,
            other,
            "add_approver",
            encode_args((other,)).unwrap(),
        )
        .expect("add_approver transport failed");
    let add_result_non_owner: Result<(), ClaimError> = decode_one(&add_res_non_owner).unwrap();
    assert_eq!(
        add_result_non_owner,
        Err(ClaimError::InsufficientPermissions)
    );

    // Owner can add approver
    let add_res_owner = pic
        .update_call(
            claim_canister,
            owner,
            "add_approver",
            encode_args((other,)).unwrap(),
        )
        .expect("add_approver transport failed");
    let add_result_owner: Result<(), ClaimError> = decode_one(&add_res_owner).unwrap();
    assert_eq!(add_result_owner, Ok(()));

    // Non-owner cannot remove approver
    let remove_res_non_owner = pic
        .update_call(
            claim_canister,
            other,
            "remove_approver",
            encode_args((other,)).unwrap(),
        )
        .expect("remove_approver transport failed");
    let remove_result_non_owner: Result<(), ClaimError> =
        decode_one(&remove_res_non_owner).unwrap();
    assert_eq!(
        remove_result_non_owner,
        Err(ClaimError::InsufficientPermissions)
    );
}

#[test]
fn test_claim_status_reverts_to_approved_on_slash_failure() {
    let (pic, claim_canister, pool_canister, owner) = setup();

    let receiver_bytes = [6u8; 29];
    let receiver = Principal::from_slice(&receiver_bytes);
    let amount = Nat::from(2000u64);
    let desc = String::from("Slash failure test");

    // Create claim
    let res = pic
        .update_call(
            claim_canister,
            owner,
            "add_claim",
            encode_args((receiver, amount, pool_canister, desc)).unwrap(),
        )
        .expect("add_claim transport failed");
    let claim_id: u64 = decode_one::<Result<u64, ClaimError>>(&res)
        .unwrap()
        .unwrap();

    // Approve claim
    let approve_res = pic
        .update_call(
            claim_canister,
            owner,
            "approve_claim",
            encode_args((claim_id,)).unwrap(),
        )
        .expect("approve transport failed");
    let approve_result: Result<(), ClaimError> = decode_one(&approve_res).unwrap();
    assert_eq!(approve_result, Ok(()));

    // Wait for timelock to expire
    let one_day_plus_grace: Duration = Duration::from_nanos(TIMELOCK_DURATION);
    pic.advance_time(one_day_plus_grace);

    // Execute claim (this will fail because pool canister doesn't implement slash properly)
    let exec_res = pic
        .update_call(
            claim_canister,
            receiver,
            "execute_claim",
            encode_args((claim_id,)).unwrap(),
        )
        .expect("execute transport failed");
    let exec_result: Result<(), ClaimError> = decode_one(&exec_res).unwrap();
    assert_eq!(
        exec_result,
        Err(ClaimError::PoolCallFailed(
            "Ok((Err(InsufficientBalance),))".to_string()
        ))
    );

    // Verify claim status reverted back to Approved
    let claim_info_res = pic
        .query_call(
            claim_canister,
            owner,
            "get_claim",
            encode_args((claim_id,)).unwrap(),
        )
        .expect("get_claim transport failed");
    let claim_info: Option<ClaimInfo> = decode_one(&claim_info_res).unwrap();
    assert!(
        claim_info.is_some(),
        "expected claim to exist after failed slash, but got None"
    );
    let claim_info = claim_info.unwrap();
    assert_eq!(claim_info.status, ClaimStatus::Approved);
}
