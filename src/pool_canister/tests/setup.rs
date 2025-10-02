use candid::{encode_args, Nat, Principal};
use pool_canister::Account;
use pocket_ic::PocketIc;

#[path = "types.rs"]
mod ledger_types;
use ledger_types::{ArchiveOptions, FeatureFlags, InitArgs, LedgerArg};


const ICRC1_LEDGER_WASM_PATH: &str = "../../ic-icrc1-ledger.wasm";
const WASM_PATH: &str = "../../target/wasm32-unknown-unknown/release/pool_canister.wasm";

pub fn setup() -> (PocketIc, Principal, Principal) {
    let pic = PocketIc::new();

    // Create and setup ICRC-1 ledger first
    let ledger_id = pic.create_canister();
    pic.add_cycles(ledger_id, 2_000_000_000_000);
    let ledger_wasm = std::fs::read(ICRC1_LEDGER_WASM_PATH).expect("ICRC-1 ledger WASM not found");

    // Setup ledger initialization
    let minting_account = Account {
        owner: ledger_id,
        subaccount: None,
    };

    // Setup multiple test users with sufficient balances
    let user1 = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap();
    let user2 = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();
    let user3 = Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap();
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();

    let initial_balances = vec![
        (
            Account {
                owner: user1,
                subaccount: None,
            },
            Nat::from(10_000_000_000u64),
        ),
        (
            Account {
                owner: user2,
                subaccount: None,
            },
            Nat::from(10_000_000_000u64),
        ),
        (
            Account {
                owner: user3,
                subaccount: None,
            },
            Nat::from(10_000_000_000u64),
        ),
        (
            Account {
                owner: user,
                subaccount: None,
            },
            Nat::from(10_000_000_000u64),
        ),
    ];

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

    // Install ledger with InitArgs wrapped in LedgerArg
    let ledger_arg = LedgerArg::Init(init_args);
    pic.install_canister(
        ledger_id,
        ledger_wasm,
        encode_args((ledger_arg,)).unwrap(),
        None,
    );

    // Create and setup main canister with ledger_id
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 2_000_000_000_000);
    let wasm = std::fs::read(WASM_PATH)
        .expect("Build first: cargo build --target wasm32-unknown-unknown --release");

    // Install canister with ledger_id as initial token_id
    let executor = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap();
    let init_args = encode_args((ledger_id, executor)).unwrap();
    pic.install_canister(canister_id, wasm, init_args, None);

    (pic, canister_id, ledger_id)
}
