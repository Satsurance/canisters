use candid::{decode_one, encode_args, Nat, Principal};
use icp_canister_backend::{types::UserDepositInfo, Account, Deposit, PoolError};
use pocket_ic::PocketIc;
use sha2::{Digest, Sha256};
mod types;
use types::*;
mod utils;
use utils::{create_deposit, TRANSFER_FEE};

const ICRC1_LEDGER_WASM_PATH: &str = "../../src/icp_canister_backend/ic-icrc1-ledger.wasm";
const WASM_PATH: &str = "../../target/wasm32-unknown-unknown/release/icp_canister_backend.wasm";

fn setup() -> (PocketIc, Principal, Principal) {
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
    let init_args = encode_args((ledger_id,)).unwrap();
    pic.install_canister(canister_id, wasm, init_args, None);

    (pic, canister_id, ledger_id)
}

#[test]
fn test_get_deposit_subaccount() {
    let (pic, canister_id, _) = setup();
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let timelock: u64 = 123456789;

    let result = pic
        .query_call(
            canister_id,
            user,
            "get_deposit_subaccount",
            encode_args((user.clone(), timelock)).unwrap(),
        )
        .expect("Failed to call get_deposit_subaccount");

    let returned_subaccount: [u8; 32] = decode_one(&result).unwrap();

    // Expected subaccount calculation
    let mut hasher = Sha256::new();
    hasher.update(user.as_slice());
    hasher.update(timelock.to_be_bytes());
    let expected_subaccount: [u8; 32] = hasher.finalize().into();

    assert_eq!(returned_subaccount, expected_subaccount);
}

#[test]
fn test_deposit_flow() {
    let (pic, canister_id, ledger_id) = setup();
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let timelock: u64 = 86400;
    let deposit_amount = Nat::from(100_000_000u64);

    create_deposit(
        &pic,
        canister_id,
        ledger_id,
        user,
        deposit_amount.clone(),
        timelock,
    );

    // Verify the canister main account has the tokens
    let main_account = Account {
        owner: canister_id,
        subaccount: None,
    };

    let balance_check = pic
        .query_call(
            ledger_id,
            user,
            "icrc1_balance_of",
            encode_args((main_account,)).unwrap(),
        )
        .expect("Failed to check canister balance");

    let canister_balance: Nat = decode_one(&balance_check).unwrap();
    let expected_balance = deposit_amount.clone() - TRANSFER_FEE.clone();
    assert_eq!(
        canister_balance, expected_balance,
        "Canister should have received the exact expected tokens"
    );
}

#[test]
fn test_deposit_fails_without_transfer() {
    let (pic, canister_id, _ledger_id) = setup();
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let timelock: u64 = 86400;
    // Directly call deposit
    let deposit_result = pic
        .update_call(
            canister_id,
            user,
            "deposit",
            encode_args((user, timelock)).unwrap(),
        )
        .expect("Failed to call deposit");

    let result: Result<(), PoolError> = decode_one(&deposit_result).unwrap();
    assert!(
        matches!(result, Err(PoolError::InsufficientBalance)),
        "Expected InsufficientBalance error, got: {:?}",
        result
    );
}
#[test]
fn test_deposit_fails_below_minimum_amount() {
    let (pic, canister_id, ledger_id) = setup();
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let timelock: u64 = 86400;

    let small_deposit_amount = Nat::from(50_000u64);

    // Get subaccount
    let subaccount_result = pic
        .query_call(
            canister_id,
            user,
            "get_deposit_subaccount",
            encode_args((user, timelock)).unwrap(),
        )
        .expect("Failed to get deposit subaccount");
    let subaccount: [u8; 32] = decode_one(&subaccount_result).unwrap();

    // Transfer small amount to subaccount
    let transfer_args = icp_canister_backend::TransferArg {
        from_subaccount: None,
        to: Account {
            owner: canister_id,
            subaccount: Some(subaccount.to_vec()),
        },
        amount: small_deposit_amount.clone(),
        fee: Some(utils::TRANSFER_FEE.clone()),
        memo: None,
        created_at_time: None,
    };
    let transfer_result = pic
        .update_call(
            ledger_id,
            user,
            "icrc1_transfer",
            encode_args((transfer_args,)).unwrap(),
        )
        .expect("Failed to transfer tokens");
    let transfer_result: utils::TransferResult = decode_one(&transfer_result).unwrap();
    assert!(
        matches!(transfer_result, utils::TransferResult::Ok(_)),
        "Transfer should succeed"
    );

    // Try to create deposit - should fail
    let deposit_result = pic
        .update_call(
            canister_id,
            user,
            "deposit",
            encode_args((user, timelock)).unwrap(),
        )
        .expect("Failed to call deposit");
    let result: Result<(), PoolError> = decode_one(&deposit_result).unwrap();
    assert!(
        matches!(result, Err(PoolError::InsufficientBalance)),
        "Expected InsufficientBalance error for deposit below minimum, got: {:?}",
        result
    );
}

#[test]
fn test_successful_withdrawal() {
    let (pic, canister_id, ledger_id) = setup();
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let timelock: u64 = 604800;
    let deposit_amount = Nat::from(100_000_000u64);

    // Check user's initial balance before deposit
    let user_account = Account {
        owner: user,
        subaccount: None,
    };
    let initial_balance_result = pic
        .query_call(
            ledger_id,
            user,
            "icrc1_balance_of",
            encode_args((user_account.clone(),)).unwrap(),
        )
        .expect("Failed to check user initial balance");
    let initial_balance: Nat = decode_one(&initial_balance_result).unwrap();

    create_deposit(
        &pic,
        canister_id,
        ledger_id,
        user,
        deposit_amount.clone(),
        timelock,
    );

    pic.advance_time(std::time::Duration::from_secs(timelock + 1));
    pic.tick();

    // Now withdraw
    let withdraw_result = pic
        .update_call(canister_id, user, "withdraw", encode_args((0u64,)).unwrap())
        .expect("Failed to call withdraw");
    let result: Result<(), PoolError> = decode_one(&withdraw_result).unwrap();
    assert!(matches!(result, Ok(_)), "Withdraw failed: {:?}", result);

    // Verify user received the tokens back
    let final_balance_result = pic
        .query_call(
            ledger_id,
            user,
            "icrc1_balance_of",
            encode_args((user_account,)).unwrap(),
        )
        .expect("Failed to check user final balance");
    let final_balance: Nat = decode_one(&final_balance_result).unwrap();

    let expected_balance = initial_balance.clone() - (TRANSFER_FEE.clone() * 3u64);
    assert_eq!(
        final_balance, expected_balance,
        "User should have received tokens back. Initial: {}, Final: {}, Expected: {}",
        initial_balance, final_balance, expected_balance
    );
}

#[test]
fn test_withdraw_invalid_principal() {
    let (pic, canister_id, ledger_id) = setup();
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let other = Principal::from_text("aaaaa-aa").unwrap();
    let timelock: u64 = 1;
    let deposit_amount = Nat::from(100_000_000u64);

    create_deposit(
        &pic,
        canister_id,
        ledger_id,
        user,
        deposit_amount.clone(),
        timelock,
    );

    // Try to withdraw as other principal
    let withdraw_result = pic
        .update_call(
            canister_id,
            other,
            "withdraw",
            encode_args((0u64,)).unwrap(),
        )
        .expect("Failed to call withdraw");
    let result: Result<(), PoolError> = decode_one(&withdraw_result).unwrap();
    assert!(
        matches!(result, Err(PoolError::NotOwner)),
        "Expected NotOwner error, got: {:?}",
        result
    );
}

#[test]
fn test_withdraw_before_timelock() {
    let (pic, canister_id, ledger_id) = setup();
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let timelock: u64 = 1000000000;
    let deposit_amount = Nat::from(100_000_000u64);

    create_deposit(
        &pic,
        canister_id,
        ledger_id,
        user,
        deposit_amount.clone(),
        timelock,
    );

    // Try to withdraw before timelock
    let withdraw_result = pic
        .update_call(canister_id, user, "withdraw", encode_args((0u64,)).unwrap())
        .expect("Failed to call withdraw");
    let result: Result<(), PoolError> = decode_one(&withdraw_result).unwrap();
    assert!(
        matches!(result, Err(PoolError::TimelockNotExpired)),
        "Expected TimelockNotExpired error, got: {:?}",
        result
    );
}

#[test]
fn test_withdraw_invalid_deposit_id() {
    let (pic, canister_id, _ledger_id) = setup();
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    // Try to withdraw with invalid deposit id
    let withdraw_result = pic
        .update_call(
            canister_id,
            user,
            "withdraw",
            encode_args((999u64,)).unwrap(),
        )
        .expect("Failed to call withdraw");
    let result: Result<(), PoolError> = decode_one(&withdraw_result).unwrap();
    assert!(
        matches!(result, Err(PoolError::NoDeposit)),
        "Expected NoDeposit error, got: {:?}",
        result
    );
}

#[test]
fn test_user_deposit_tracking() {
    let (pic, canister_id, ledger_id) = setup();
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let timelock: u64 = 86400;
    let deposit_amount = Nat::from(100_000_000u64);

    // Check initial user deposits (should be empty)
    let initial_deposits_result = pic
        .query_call(
            canister_id,
            user,
            "get_user_deposits",
            encode_args((user,)).unwrap(),
        )
        .expect("Failed to get user deposits");
    let initial_deposits: Vec<UserDepositInfo> = decode_one(&initial_deposits_result).unwrap();
    assert_eq!(
        initial_deposits.len(),
        0,
        "User should have no deposits initially"
    );

    // Create first deposit
    create_deposit(
        &pic,
        canister_id,
        ledger_id,
        user,
        deposit_amount.clone(),
        timelock,
    );

    // Check user deposits after first deposit
    let deposits_after_first_result = pic
        .query_call(
            canister_id,
            user,
            "get_user_deposits",
            encode_args((user,)).unwrap(),
        )
        .expect("Failed to get user deposits");
    let deposits_after_first: Vec<UserDepositInfo> =
        decode_one(&deposits_after_first_result).unwrap();
    assert_eq!(deposits_after_first.len(), 1, "User should have 1 deposit");
    assert_eq!(
        deposits_after_first[0].deposit_id, 0,
        "First deposit should have ID 0"
    );
    assert_eq!(
        deposits_after_first[0].amount,
        deposit_amount.clone() - TRANSFER_FEE.clone(),
        "First deposit should have correct amount"
    );

    let first_deposit_time = deposits_after_first[0].unlock_time - (timelock * 1_000_000_000);
    assert_eq!(
        deposits_after_first[0].unlock_time,
        first_deposit_time + (timelock * 1_000_000_000),
        "First deposit should have correct unlock time"
    );

    // Create second deposit
    create_deposit(
        &pic,
        canister_id,
        ledger_id,
        user,
        deposit_amount.clone(),
        timelock,
    );

    // Check user deposits after second deposit
    let deposits_after_second_result = pic
        .query_call(
            canister_id,
            user,
            "get_user_deposits",
            encode_args((user,)).unwrap(),
        )
        .expect("Failed to get user deposits");
    let deposits_after_second: Vec<UserDepositInfo> =
        decode_one(&deposits_after_second_result).unwrap();
    assert_eq!(
        deposits_after_second.len(),
        2,
        "User should have 2 deposits"
    );
    assert_eq!(
        deposits_after_second[0].deposit_id, 0,
        "First deposit should have ID 0"
    );
    assert_eq!(
        deposits_after_second[0].amount,
        deposit_amount.clone() - TRANSFER_FEE.clone(),
        "First deposit should have correct amount"
    );
    assert_eq!(
        deposits_after_second[0].unlock_time,
        first_deposit_time + (timelock * 1_000_000_000),
        "First deposit should have correct unlock time"
    );
    assert_eq!(
        deposits_after_second[1].deposit_id, 1,
        "Second deposit should have ID 1"
    );
    assert_eq!(
        deposits_after_second[1].amount,
        deposit_amount.clone() - TRANSFER_FEE.clone(),
        "Second deposit should have correct amount"
    );

    let second_deposit_time = deposits_after_second[1].unlock_time - (timelock * 1_000_000_000);
    assert_eq!(
        deposits_after_second[1].unlock_time,
        second_deposit_time + (timelock * 1_000_000_000),
        "Second deposit should have correct unlock time"
    );

    // Withdraw first deposit
    pic.advance_time(std::time::Duration::from_secs(timelock + 1));
    pic.tick();

    let withdraw_result = pic
        .update_call(canister_id, user, "withdraw", encode_args((0u64,)).unwrap())
        .expect("Failed to call withdraw");
    let result: Result<(), PoolError> = decode_one(&withdraw_result).unwrap();
    assert!(matches!(result, Ok(_)), "Withdraw failed: {:?}", result);

    // Check user deposits after withdrawal
    let deposits_after_withdraw_result = pic
        .query_call(
            canister_id,
            user,
            "get_user_deposits",
            encode_args((user,)).unwrap(),
        )
        .expect("Failed to get user deposits");
    let deposits_after_withdraw: Vec<UserDepositInfo> =
        decode_one(&deposits_after_withdraw_result).unwrap();
    assert_eq!(
        deposits_after_withdraw.len(),
        1,
        "User should have 1 deposit after withdrawal"
    );
    assert_eq!(
        deposits_after_withdraw[0].deposit_id, 1,
        "Remaining deposit should have ID 1"
    );
}

#[test]
fn test_get_deposit() {
    let (pic, canister_id, ledger_id) = setup();
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let timelock: u64 = 86400;
    let deposit_amount = Nat::from(100_000_000u64);

    // Try to get non-existent deposit
    let get_deposit_result = pic
        .query_call(
            canister_id,
            user,
            "get_deposit",
            encode_args((999u64,)).unwrap(),
        )
        .expect("Failed to call get_deposit");
    let non_existent_deposit: Option<Deposit> = decode_one(&get_deposit_result).unwrap();
    assert!(
        non_existent_deposit.is_none(),
        "Non-existent deposit should return None"
    );

    // Create a deposit
    create_deposit(
        &pic,
        canister_id,
        ledger_id,
        user,
        deposit_amount.clone(),
        timelock,
    );

    // Get the created deposit
    let get_deposit_result = pic
        .query_call(
            canister_id,
            user,
            "get_deposit",
            encode_args((0u64,)).unwrap(),
        )
        .expect("Failed to call get_deposit");
    let deposit: Option<Deposit> = decode_one(&get_deposit_result).unwrap();

    assert!(deposit.is_some(), "Deposit should exist");
    let deposit = deposit.unwrap();
    assert_eq!(
        deposit.amount,
        deposit_amount.clone() - TRANSFER_FEE.clone(),
        "Deposit should have correct amount"
    );

    // Verify unlock time is in the future and reasonable
    assert!(
        deposit.unlocktime > 0,
        "Deposit unlock time should be positive"
    );
    assert!(
        deposit.unlocktime > timelock * 1_000_000_000,
        "Deposit unlock time should be at least timelock seconds in the future"
    );
}
