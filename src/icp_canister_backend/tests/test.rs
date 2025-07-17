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
fn test_add() {
    let (pic, canister_id) = setup();
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    
    let result = pic.query_call(canister_id, user, "add", encode_args((5i32, 3i32)).unwrap())
        .expect("Failed to call add");
    
    let sum: i32 = decode_one(&result).unwrap();
    assert_eq!(sum, 8);
}