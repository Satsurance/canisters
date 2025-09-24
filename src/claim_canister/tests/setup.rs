use candid::{encode_args, Principal};
use pocket_ic::PocketIc;

const CLAIM_WASM_PATH: &str = "../../target/wasm32-unknown-unknown/release/claim_canister.wasm";
const POOL_WASM_PATH: &str = "../../target/wasm32-unknown-unknown/release/pool_canister.wasm";

pub fn setup() -> (PocketIc, Principal, Principal, Principal) {
    let pic = PocketIc::new();
    
    let owner_bytes = [1u8; 29];
    let owner = Principal::from_slice(&owner_bytes);
    
    let token_bytes = [2u8; 29];
    let token_id = Principal::from_slice(&token_bytes);

    let claim_wasm = std::fs::read(CLAIM_WASM_PATH)
        .expect("Build first: cargo build --target wasm32-unknown-unknown --release -p claim_canister");
    let claim_canister = pic.create_canister();
    pic.add_cycles(claim_canister, 2_000_000_000_000);
    pic.install_canister(claim_canister, claim_wasm, encode_args((owner,)).unwrap(), None);

    let pool_wasm = std::fs::read(POOL_WASM_PATH)
        .expect("Build first: cargo build --target wasm32-unknown-unknown --release -p pool_canister");
    let pool_canister = pic.create_canister();
    pic.add_cycles(pool_canister, 2_000_000_000_000);
    
    pic.install_canister(
        pool_canister, 
        pool_wasm, 
        encode_args((token_id, claim_canister)).unwrap(),
        None
    );

    (pic, claim_canister, pool_canister, owner)
}
