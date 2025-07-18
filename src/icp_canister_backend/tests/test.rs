use candid::{decode_one, encode_args, Principal};
use pocket_ic::PocketIc;

const WASM_PATH: &str = "../../target/wasm32-unknown-unknown/release/icp_canister_backend.wasm";

fn setup() -> (PocketIc, Principal) {
    let pic = PocketIc::new();
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 2_000_000_000_000);
    let wasm = std::fs::read(WASM_PATH).expect("Build first: cargo build --target wasm32-unknown-unknown --release");
    pic.install_canister(canister_id, wasm, vec![], None);
    (pic, canister_id)
}

#[test]
fn add() {
    let (pic, canister_id) = setup();
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    
    let result = pic.query_call(canister_id, user, "add", encode_args((5i32, 3i32)).unwrap())
        .expect("Failed to call add");
    
    let sum: i32 = decode_one(&result).unwrap();
    assert_eq!(sum, 8);
}

#[test]
fn get_deposit_subaccount() {
    let (pic, canister_id) = setup();
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let timelock = 86400u64; // 1 day
    
    let result = pic.query_call(
        canister_id,
        Principal::anonymous(),
        "get_deposit_subaccount",
        encode_args((user, timelock)).unwrap(),
    ).expect("Failed to call get_deposit_subaccount");
    
    let subaccount: [u8; 32] = decode_one(&result).unwrap();
    assert_eq!(subaccount.len(), 32);
    
    println!("Generated subaccount: {:?}", subaccount);
}

#[test]
fn subaccount_deterministic() {
    let (pic, canister_id) = setup();
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let timelock = 86400u64;
    
    // Call multiple times and verify same subaccount
    let mut subaccounts = Vec::new();
    for _ in 0..5 {
        let result = pic.query_call(
            canister_id,
            Principal::anonymous(),
            "get_deposit_subaccount",
            encode_args((user, timelock)).unwrap(),
        ).expect("Failed to call get_deposit_subaccount");
        
        let subaccount: [u8; 32] = decode_one(&result).unwrap();
        subaccounts.push(subaccount);
    }
    
    // All subaccounts should be identical
    for i in 1..subaccounts.len() {
        assert_eq!(subaccounts[0], subaccounts[i]);
    }
    
    println!("Deterministic test passed");
}

#[test]
fn different_users_different_subaccounts() {
    let (pic, canister_id) = setup();
    let user1 = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let user2 = Principal::from_text("be2us-64aaa-aaaaa-qaabq-cai").unwrap();
    let timelock = 86400u64;
    
    let result1 = pic.query_call(
        canister_id,
        Principal::anonymous(),
        "get_deposit_subaccount",
        encode_args((user1, timelock)).unwrap(),
    ).expect("Failed to call get_deposit_subaccount");
    
    let result2 = pic.query_call(
        canister_id,
        Principal::anonymous(),
        "get_deposit_subaccount",
        encode_args((user2, timelock)).unwrap(),
    ).expect("Failed to call get_deposit_subaccount");
    
    let subaccount1: [u8; 32] = decode_one(&result1).unwrap();
    let subaccount2: [u8; 32] = decode_one(&result2).unwrap();
    
    assert_ne!(subaccount1, subaccount2);
    println!("Different users have different subaccounts");
}