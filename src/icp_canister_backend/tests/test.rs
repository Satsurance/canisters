use candid::{decode_one, encode_args, Principal, CandidType, Deserialize, Nat};
use pocket_ic::PocketIc;
use sha2::{Digest, Sha256};

const ICRC1_LEDGER_WASM_PATH: &str = "../../src/icp_canister_backend/ic-icrc1-ledger.wasm";
const WASM_PATH: &str = "../../target/wasm32-unknown-unknown/release/icp_canister_backend.wasm";


#[derive(CandidType, Deserialize, Debug)]
struct Account {
    owner: Principal,
    subaccount: Option<Vec<u8>>,
}

#[derive(CandidType, Deserialize, Debug)]
struct TransferArg {
    from_subaccount: Option<Vec<u8>>,
    to: Account,
    amount: Nat,
    fee: Option<Nat>,
    memo: Option<Vec<u8>>,
    created_at_time: Option<u64>,
}

#[derive(CandidType, Deserialize, Debug)]
enum TransferResult {
    Ok(Nat),
    Err(TransferError),
}

#[derive(CandidType, Deserialize, Debug)]
enum TransferError {
    BadFee { expected_fee: Nat },
    BadBurn { min_burn_amount: Nat },
    InsufficientFunds { balance: Nat },
    TooOld,
    CreatedInFuture { ledger_time: u64 },
    TemporarilyUnavailable,
    Duplicate { duplicate_of: Nat },
    GenericError { error_code: Nat, message: String },
}

#[derive(CandidType, Deserialize, Debug)]
enum DepositError {
    NoDeposit,
    InsufficientBalance,
    InvalidTimelock,
    TransferFailed,
    LedgerCallFailed,
    InternalError,
    LedgerNotSet,
    DepositAlreadyExists,
}

// Ledger initialization types
#[derive(CandidType, Deserialize)]
struct InitArgs {
    minting_account: Account,
    fee_collector_account: Option<Account>,
    transfer_fee: Nat,
    decimals: Option<u8>,
    max_memo_length: Option<u16>,
    token_symbol: String,
    token_name: String,
    metadata: Vec<(String, MetadataValue)>,
    initial_balances: Vec<(Account, Nat)>,
    feature_flags: Option<FeatureFlags>,
    maximum_number_of_accounts: Option<u64>,
    accounts_overflow_trim_quantity: Option<u64>,
    archive_options: ArchiveOptions,
}

#[derive(CandidType, Deserialize)]
enum MetadataValue {
    Nat(Nat),
    Int(i64),
    Text(String),
    Blob(Vec<u8>),
}

#[derive(CandidType, Deserialize)]
struct FeatureFlags {
    icrc2: bool,
}

#[derive(CandidType, Deserialize)]
struct ArchiveOptions {
    num_blocks_to_archive: u64,
    max_transactions_per_response: Option<u64>,
    trigger_threshold: u64,
    max_message_size_bytes: Option<u64>,
    cycles_for_archive_creation: Option<u64>,
    node_max_memory_size_bytes: Option<u64>,
    controller_id: Principal,
    more_controller_ids: Option<Vec<Principal>>,
}

#[derive(CandidType, Deserialize)]
enum LedgerArg {
    Init(InitArgs),
    Upgrade(Option<()>),
}

fn setup() -> (PocketIc, Principal) {
    let pic = PocketIc::new();
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 2_000_000_000_000);
    let wasm = std::fs::read(WASM_PATH).expect("Build first: cargo build --target wasm32-unknown-unknown --release");
    pic.install_canister(canister_id, wasm, vec![], None);
    (pic, canister_id)
}

fn setup_with_ledger() -> (PocketIc, Principal, Principal) {
    let pic = PocketIc::new();
    
    // Create and setup main canister
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 2_000_000_000_000);
    let wasm = std::fs::read(WASM_PATH).expect("Build first: cargo build --target wasm32-unknown-unknown --release");
    pic.install_canister(canister_id, wasm, vec![], None);
    
    // Create and setup ICRC-1 ledger
    let ledger_id = pic.create_canister();
    pic.add_cycles(ledger_id, 2_000_000_000_000);
    let ledger_wasm = std::fs::read(ICRC1_LEDGER_WASM_PATH).expect("ICRC-1 ledger WASM not found");
    
    // Setup ledger initialization
    let minting_account = Account {
        owner: ledger_id,
        subaccount: None,
    };
    
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let initial_balances = vec![(
        Account {
            owner: user,
            subaccount: None,
        },
        Nat::from(1_000_000_000u64), // 1000 tokens (assuming 6 decimals)
    )];
    
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
    
    (pic, canister_id, ledger_id)
}

#[test]
fn test_get_deposit_subaccount() {
    let (pic, canister_id) = setup();
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let timelock: u64 = 123456789;

    let result = pic.query_call(canister_id, user, "get_deposit_subaccount", encode_args((user.clone(), timelock)).unwrap())
        .expect("Failed to call get_deposit_subaccount");

    let returned_subaccount: [u8; 32] = decode_one(&result).unwrap();

 
    let mut hasher = Sha256::new();
    hasher.update(user.as_slice());
    hasher.update(timelock.to_be_bytes());
    let expected_subaccount: [u8; 32] = hasher.finalize().into();

    assert_eq!(returned_subaccount, expected_subaccount);

}

#[test]
fn test_deposit_flow() {
    let (pic, canister_id, ledger_id) = setup_with_ledger();
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let timelock: u64 = 86400; // 1 day in seconds
    let deposit_amount = 100_000_000u64; // 100 tokens
    
    // Set the ledger canister ID
    let _set_ledger_result = pic.update_call(
        canister_id,
        user,
        "set_ledger_canister_id",
        encode_args((ledger_id,)).unwrap(),
    ).expect("Failed to set ledger canister ID");
    
    println!("Ledger canister ID set: {}", ledger_id);

    //  User calls get_deposit_subaccount
    let subaccount_result = pic.query_call(
        canister_id, 
        user, 
        "get_deposit_subaccount", 
        encode_args((user, timelock)).unwrap()
    ).expect("Failed to get deposit subaccount");
    
    let subaccount: [u8; 32] = decode_one(&subaccount_result).unwrap();
    println!("Generated subaccount: {}", hex::encode(&subaccount));
    
    //  User transfers tokens to the subaccount
    let transfer_args = TransferArg {
        from_subaccount: None,
        to: Account {
            owner: canister_id,
            subaccount: Some(subaccount.to_vec()),
        },
        amount: Nat::from(deposit_amount),
        fee: Some(Nat::from(10_000u64)),
        memo: Some(b"deposit".to_vec()),
        created_at_time: None,
    };
    
    let transfer_result = pic.update_call(
        ledger_id,
        user,
        "icrc1_transfer",
        encode_args((transfer_args,)).unwrap(),
    ).expect("Failed to transfer tokens");
    
    let transfer_result: TransferResult = decode_one(&transfer_result).unwrap();
    match transfer_result {
        TransferResult::Ok(block_index) => {
            println!("Transfer successful, block index: {}", block_index);
        },
        TransferResult::Err(e) => {
            panic!(" Transfer failed: {:?}", e);
        }
    }
    
    //  User calls deposit function
    let deposit_result = pic.update_call(
        canister_id,
        user,
        "deposit",
        encode_args((user, timelock)).unwrap(),
    ).expect("Failed to call deposit");
    
    let result: Result<(), DepositError> = decode_one(&deposit_result).unwrap();
    match result {
        Ok(()) => {
            println!("Deposit successful!");
        },
        Err(e) => {
            panic!(" Deposit failed: {:?}", e);
        }
    }
    
  
}

