use candid::{decode_one, encode_args, Principal};
use pocket_ic::PocketIc;
use sha2::{Digest, Sha256};

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
fn test_get_deposit_subaccount() {
    let (pic, canister_id) = setup();
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let timelock: u64 = 123456789;

    let result = pic.query_call(canister_id, user, "get_deposit_subaccount", encode_args((user.clone(), timelock)).unwrap())
        .expect("Failed to call get_deposit_subaccount");

    let returned_subaccount: [u8; 32] = decode_one(&result).unwrap();

    // Calculate expected hash in the same way as in the canister
    let mut hasher = Sha256::new();
    hasher.update(user.as_slice());
    hasher.update(timelock.to_be_bytes());
    let expected_subaccount: [u8; 32] = hasher.finalize().into();

    assert_eq!(returned_subaccount, expected_subaccount);
}