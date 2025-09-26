use candid::{encode_args, Decode, Nat, Principal};
use pool_canister::types::{Account, TransferArg};
use pocket_ic::PocketIc;

#[path = "utils.rs"]
mod utils;
use self::utils::get_stakable_episode;

#[path = "types.rs"]
mod ledger_types;
use ledger_types::{ArchiveOptions, FeatureFlags, InitArgs, LedgerArg};

const CLAIM_WASM_PATH: &str = "../../target/wasm32-unknown-unknown/release/claim_canister.wasm";
const POOL_WASM_PATH: &str = "../../target/wasm32-unknown-unknown/release/pool_canister.wasm";
const ICRC1_LEDGER_WASM_PATH: &str = "../../ic-icrc1-ledger.wasm";

pub fn setup() -> (PocketIc, Principal, Principal, Principal) {
    let pic = PocketIc::new();
    
    let owner_bytes = [1u8; 29];
    let owner = Principal::from_slice(&owner_bytes);
    
    // Install ICRC-1 ledger and fund owner
    let ledger_id = pic.create_canister();
    pic.add_cycles(ledger_id, 2_000_000_000_000);
    let ledger_wasm = std::fs::read(ICRC1_LEDGER_WASM_PATH)
        .expect("ICRC-1 ledger WASM not found. Build/download the test wasm.");

    let minting_account = Account { owner: ledger_id, subaccount: None };
    let owner_account = Account { owner, subaccount: None };
    let initial_balances = vec![(owner_account, Nat::from(10_000_000_000_000u64))];

    let init_args = InitArgs {
        minting_account,
        fee_collector_account: None,
        transfer_fee: Nat::from(10_000u64),
        decimals: Some(6),
        max_memo_length: Some(64),
        token_symbol: "TEST".to_string(),
        token_name: "Test Token".to_string(),
        metadata: vec![],
        initial_balances,
        feature_flags: Some(FeatureFlags { icrc2: true }),
        maximum_number_of_accounts: None,
        accounts_overflow_trim_quantity: None,
        archive_options: ArchiveOptions {
            num_blocks_to_archive: 1000,
            max_transactions_per_response: Some(100),
            trigger_threshold: 2000,
            max_message_size_bytes: Some(1024 * 1024),
            cycles_for_archive_creation: Some(1_000_000_000_000),
            node_max_memory_size_bytes: Some(32 * 1024 * 1024),
            controller_id: ledger_id,
            more_controller_ids: None,
        },
    };
    let ledger_arg = LedgerArg::Init(init_args);
    pic.install_canister(ledger_id, ledger_wasm, encode_args((ledger_arg,)).unwrap(), None);

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
        encode_args((ledger_id, claim_canister)).unwrap(),
        None
    );

    // Transfer funds to pool canister for slashing
    let transfer_to_pool = TransferArg {
        from_subaccount: None,
        to: Account { owner: pool_canister, subaccount: None },
        amount: Nat::from(1_000_000_000_000u64),
        fee: Some(Nat::from(10_000u64)),
        memo: None,
        created_at_time: None,
    };
    
    pic.update_call(
        ledger_id,
        owner,
        "icrc1_transfer",
        encode_args((transfer_to_pool,)).unwrap(),
    ).unwrap();

    // Fund pool with deposit
    let current_episode: u64 = get_stakable_episode(&pic, pool_canister, owner);
    
    let sub_bytes = pic
        .query_call(
            pool_canister,
            owner,
            "get_deposit_subaccount",
            encode_args((owner, current_episode)).unwrap(),
        )
        .unwrap();
    let subaccount: [u8; 32] = Decode!(&sub_bytes, [u8; 32]).unwrap();

    let transfer_args = TransferArg {
        from_subaccount: None,
        to: Account { owner: pool_canister, subaccount: Some(subaccount.to_vec()) },
        amount: Nat::from(100_000_000_000u64),
        fee: Some(Nat::from(10_000u64)),
        memo: None,
        created_at_time: None,
    };
    
    pic.update_call(
        ledger_id,
        owner,
        "icrc1_transfer",
        encode_args((transfer_args,)).unwrap(),
    ).unwrap();

    // Complete deposit
    pic.update_call(
        pool_canister,
        owner,
        "deposit",
        encode_args((owner, current_episode)).unwrap(),
    ).unwrap();

    (pic, claim_canister, pool_canister, owner)
}
