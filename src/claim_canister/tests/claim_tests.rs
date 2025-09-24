mod setup;
use candid::{encode_args, Nat, Principal, Decode};
use setup::setup;
use claim_canister::types::{ClaimError, ClaimInfo, ClaimStatus};

#[test]
fn test_claim_workflow_with_real_pool_canister() {
    let (pic, claim_canister, pool_canister, owner) = setup();

    let receiver_bytes = [3u8; 29];
    let receiver = Principal::from_slice(&receiver_bytes);
    let amount = Nat::from(1000u64); 
    let desc = String::from("Insurance payout for property damage");
    
    let res = pic.update_call(
        claim_canister,
        owner,
        "add_claim",
        encode_args((receiver, amount.clone(), pool_canister, desc)).unwrap(),
    ).expect("Failed to add claim");
    
    let decoded: Result<Result<u64, ClaimError>, _> = Decode!(&res, Result<u64, ClaimError>);
    let claim_id = decoded.expect("Failed to decode claim_id").expect("add_claim returned Err");
    println!("Created claim with ID: {}", claim_id);

    let approve_res = pic.update_call(
        claim_canister,
        owner,
        "approve_claim",
        encode_args((claim_id,)).unwrap(),
    ).expect("Approval transport failed");
    let approve_decoded: Result<Result<(), ClaimError>, _> = Decode!(&approve_res, Result<(), ClaimError>);
    assert!(approve_decoded.is_ok());
    assert!(approve_decoded.unwrap().is_ok());

    // Verify claim is approved before timelock
    let claim_info_res = pic.query_call(
        claim_canister,
        owner,
        "get_claim",
        encode_args((claim_id,)).unwrap(),
    ).expect("get_claim transport failed");
    let claim_info: Option<ClaimInfo> = Decode!(&claim_info_res, Option<ClaimInfo>).expect("decode ClaimInfo failed");
    assert!(claim_info.is_some());
    let claim_info = claim_info.unwrap();
    assert_eq!(claim_info.status, ClaimStatus::Approved);
    assert_eq!(claim_info.can_execute, false);

    pic.advance_time(std::time::Duration::from_secs(24 * 60 * 60 + 60));

    // execute_claim
    let exec_res = pic.update_call(
        claim_canister,
        receiver,
        "execute_claim", 
        encode_args((claim_id,)).unwrap(),
    ).expect("execute_claim transport failed");
    let exec_decoded: Result<Result<(), ClaimError>, _> = Decode!(&exec_res, Result<(), ClaimError>);
    assert!(exec_decoded.is_ok());
    if let Ok(inner) = exec_decoded { assert_eq!(inner, Err(ClaimError::PoolCallFailed)); }

    // Verify claim remains approved
    let final_claim_res = pic.query_call(
        claim_canister,
        owner,
        "get_claim", 
        encode_args((claim_id,)).unwrap(),
    ).expect("final get_claim transport failed");
    let final_info: Option<ClaimInfo> = Decode!(&final_claim_res, Option<ClaimInfo>).expect("decode final ClaimInfo failed");
    assert!(final_info.is_some());
    let final_info = final_info.unwrap();
    assert_eq!(final_info.status, ClaimStatus::Approved);
}
