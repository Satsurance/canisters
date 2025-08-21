use candid::{decode_one, encode_args, Nat, Principal};
use icp_canister_backend::{types::UserDepositInfo, Account, Deposit, PoolError, PoolState};
use pocket_ic::PocketIc;
use sha2::{Digest, Sha256};
mod types;
use types::*;
mod utils;
use utils::{
    advance_time, create_deposit, get_current_episode, get_episode_time_to_end,
    get_stakable_episode, reward_pool, TRANSFER_FEE,assert_with_error
};

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
    let executor = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap();
    let init_args = encode_args((ledger_id, executor)).unwrap();
    pic.install_canister(canister_id, wasm, init_args, None);

    (pic, canister_id, ledger_id)
}

#[test]
fn test_get_deposit_subaccount() {
    let (pic, canister_id, _) = setup();
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let episode: u64 = 123456789;

    let result = pic
        .query_call(
            canister_id,
            user,
            "get_deposit_subaccount",
            encode_args((user.clone(), episode)).unwrap(),
        )
        .expect("Failed to call get_deposit_subaccount");

    let returned_subaccount: [u8; 32] = decode_one(&result).unwrap();

    // Expected subaccount calculation
    let mut hasher = Sha256::new();
    hasher.update(user.as_slice());
    hasher.update(episode.to_be_bytes());
    let expected_subaccount: [u8; 32] = hasher.finalize().into();

    assert_eq!(returned_subaccount, expected_subaccount);
}

#[test]
fn test_deposit_flow() {
    let (pic, canister_id, ledger_id) = setup();
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let deposit_amount = Nat::from(100_000_000u64);

    let current_episode = get_stakable_episode(&pic, canister_id, 0);

    create_deposit(
        &pic,
        canister_id,
        ledger_id,
        user,
        deposit_amount.clone(),
        current_episode,
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

    // Check pool state after deposit
    let pool_state_result = pic
        .query_call(
            canister_id,
            user,
            "get_pool_state",
            encode_args(()).unwrap(),
        )
        .expect("Failed to get pool state");
    let pool_state: PoolState = decode_one(&pool_state_result).unwrap();

    let expected_assets = deposit_amount.clone() - TRANSFER_FEE.clone();
    assert_eq!(
        pool_state.total_assets, expected_assets,
        "Pool should have correct total assets"
    );
    assert_eq!(
        pool_state.total_shares, expected_assets,
        "First deposit should have 1:1 share ratio"
    );
}

#[test]
fn test_deposit_fails_without_transfer() {
    let (pic, canister_id, _ledger_id) = setup();
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();

    let current_episode = get_stakable_episode(&pic, canister_id, 0);

    // Directly call deposit without transferring tokens first
    let deposit_result = pic
        .update_call(
            canister_id,
            user,
            "deposit",
            encode_args((user, current_episode)).unwrap(),
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

    let current_episode = get_stakable_episode(&pic, canister_id, 0);

    let small_deposit_amount = Nat::from(50_000u64);

    // Get subaccount
    let subaccount_result = pic
        .query_call(
            canister_id,
            user,
            "get_deposit_subaccount",
            encode_args((user, current_episode)).unwrap(),
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
            encode_args((user, current_episode)).unwrap(),
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

    // Create deposit and advance time to simulate finished episode
    let current_episode = get_stakable_episode(&pic, canister_id, 0);
    create_deposit(
        &pic,
        canister_id,
        ledger_id,
        user,
        deposit_amount.clone(),
        current_episode,
    );
    let current_episode_time_to_end = get_episode_time_to_end(&pic, current_episode);
    advance_time(&pic, current_episode_time_to_end);

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

    // Check pool state after withdrawal
    let pool_state_result = pic
        .query_call(
            canister_id,
            user,
            "get_pool_state",
            encode_args(()).unwrap(),
        )
        .expect("Failed to get pool state");
    let pool_state: PoolState = decode_one(&pool_state_result).unwrap();

    assert_eq!(
        pool_state.total_assets,
        Nat::from(0u64),
        "Pool should have no assets after withdrawal"
    );
    assert_eq!(
        pool_state.total_shares,
        Nat::from(0u64),
        "Pool should have no shares after withdrawal"
    );
}

#[test]
fn test_withdraw_invalid_principal() {
    let (pic, canister_id, ledger_id) = setup();
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let other = Principal::from_text("aaaaa-aa").unwrap();
    let deposit_amount = Nat::from(100_000_000u64);

    let current_episode = get_stakable_episode(&pic, canister_id, 0);

    create_deposit(
        &pic,
        canister_id,
        ledger_id,
        user,
        deposit_amount.clone(),
        current_episode,
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
    let deposit_amount = Nat::from(100_000_000u64);

    let current_episode = get_stakable_episode(&pic, canister_id, 0);

    create_deposit(
        &pic,
        canister_id,
        ledger_id,
        user,
        deposit_amount.clone(),
        current_episode,
    );

    // Try to withdraw before episode ends
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

    let first_episode = get_stakable_episode(&pic, canister_id, 0);

    // Create first deposit
    create_deposit(
        &pic,
        canister_id,
        ledger_id,
        user,
        deposit_amount.clone(),
        first_episode,
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

    let expected_amount = deposit_amount.clone() - TRANSFER_FEE.clone();
    assert_eq!(
        deposits_after_first[0].shares, expected_amount,
        "First deposit should have correct shares"
    );
    assert_eq!(
        deposits_after_first[0].amount, expected_amount,
        "First deposit amount should equal shares initially"
    );
    assert_eq!(
        deposits_after_first[0].episode, first_episode,
        "First deposit should have correct episode"
    );
    // Create second deposit in next episode
    let second_episode = get_stakable_episode(&pic, canister_id, 1);
    create_deposit(
        &pic,
        canister_id,
        ledger_id,
        user,
        deposit_amount.clone(),
        second_episode,
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
        deposits_after_second[1].deposit_id, 1,
        "Second deposit should have ID 1"
    );
    assert_eq!(
        deposits_after_second[1].shares, expected_amount,
        "Second deposit should have proportional shares"
    );

    let third_episode = get_stakable_episode(&pic, canister_id, 2);
    create_deposit(
        &pic,
        canister_id,
        ledger_id,
        user,
        deposit_amount.clone(),
        third_episode,
    );

    let third_episode_time_to_end = get_episode_time_to_end(&pic, third_episode);
    advance_time(&pic, third_episode_time_to_end);

    let withdraw_result = pic
        .update_call(canister_id, user, "withdraw", encode_args((2u64,)).unwrap())
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
        2,
        "User should have 2 deposits after withdrawal (2 remaining)"
    );
}

#[test]
fn test_get_deposit() {
    let (pic, canister_id, ledger_id) = setup();
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
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

    let current_episode = get_stakable_episode(&pic, canister_id, 0);

    // Create a deposit
    create_deposit(
        &pic,
        canister_id,
        ledger_id,
        user,
        deposit_amount.clone(),
        current_episode,
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

    let expected_amount = deposit_amount.clone() - TRANSFER_FEE.clone();
    assert_eq!(
        deposit.shares, expected_amount,
        "Deposit should have correct shares"
    );
    assert_eq!(
        deposit.episode, current_episode,
        "Deposit should have correct episode"
    );
}

#[test]
fn test_shares_calculation() {
    let (pic, canister_id, ledger_id) = setup();
    let user1 = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();

    let current_episode = get_stakable_episode(&pic, canister_id, 0);

    let deposit_amount = Nat::from(200_000_000u64);

    // First user deposits
    create_deposit(
        &pic,
        canister_id,
        ledger_id,
        user1,
        deposit_amount.clone(),
        current_episode,
    );

    let pool_state_result = pic
        .query_call(
            canister_id,
            user1,
            "get_pool_state",
            encode_args(()).unwrap(),
        )
        .expect("Failed to get pool state");
    let pool_state: PoolState = decode_one(&pool_state_result).unwrap();

    let expected_amount = deposit_amount.clone() - TRANSFER_FEE.clone();
    assert_eq!(
        pool_state.total_assets, expected_amount,
        "Pool should have correct assets"
    );
    assert_eq!(
        pool_state.total_shares, expected_amount,
        "Pool should have correct shares"
    );

    // Create a second deposit from the same user to test proportional shares
    let next_episode = get_stakable_episode(&pic, canister_id, 1);
    create_deposit(
        &pic,
        canister_id,
        ledger_id,
        user1,
        deposit_amount.clone(),
        next_episode,
    );

    let pool_state_after_second = pic
        .query_call(
            canister_id,
            user1,
            "get_pool_state",
            encode_args(()).unwrap(),
        )
        .expect("Failed to get pool state");
    let pool_state_after: PoolState = decode_one(&pool_state_after_second).unwrap();

    let expected_total_assets = (deposit_amount.clone() - TRANSFER_FEE.clone()) * 2u64;
    let expected_total_shares = (deposit_amount.clone() - TRANSFER_FEE.clone()) * 2u64;

    assert_eq!(
        pool_state_after.total_assets, expected_total_assets,
        "Pool should have doubled assets"
    );
    assert_eq!(
        pool_state_after.total_shares, expected_total_shares,
        "Pool should have doubled shares"
    );

    // Check both deposits have equal shares since they were equal amounts
    let user1_deposits = pic
        .query_call(
            canister_id,
            user1,
            "get_user_deposits",
            encode_args((user1,)).unwrap(),
        )
        .expect("Failed to get user1 deposits");
    let user1_deposits: Vec<UserDepositInfo> = decode_one(&user1_deposits).unwrap();

    assert_eq!(user1_deposits.len(), 2, "User should have 2 deposits");
    assert_eq!(
        user1_deposits[0].shares, user1_deposits[1].shares,
        "Both deposits should have equal shares"
    );
    assert_eq!(
        user1_deposits[0].amount, user1_deposits[1].amount,
        "Both deposits should have equal amount"
    );

    let expected_shares = deposit_amount.clone() - TRANSFER_FEE.clone();
    assert_eq!(
        user1_deposits[0].shares, expected_shares,
        "First deposit should have expected shares"
    );
    assert_eq!(
        user1_deposits[1].shares, expected_shares,
        "Second deposit should have expected shares"
    );
}

#[test]
fn test_deposit_episode_validation() {
    let (pic, canister_id, ledger_id) = setup();
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let deposit_amount = Nat::from(100_000_000u64);

    let current_episode = get_current_episode(&pic, canister_id);

    // Test deposit in past episode (should fail - time validation)
    if current_episode > 0 {
        let past_episode = current_episode - 1;
        let subaccount_result = pic
            .query_call(
                canister_id,
                user,
                "get_deposit_subaccount",
                encode_args((user, past_episode)).unwrap(),
            )
            .expect("Failed to get deposit subaccount");
        let subaccount: [u8; 32] = decode_one(&subaccount_result).unwrap();

        let transfer_args = icp_canister_backend::TransferArg {
            from_subaccount: None,
            to: Account {
                owner: canister_id,
                subaccount: Some(subaccount.to_vec()),
            },
            amount: deposit_amount.clone(),
            fee: Some(TRANSFER_FEE.clone()),
            memo: None,
            created_at_time: None,
        };

        pic.update_call(
            ledger_id,
            user,
            "icrc1_transfer",
            encode_args((transfer_args,)).unwrap(),
        )
        .expect("Failed to transfer tokens");

        let deposit_result = pic
            .update_call(
                canister_id,
                user,
                "deposit",
                encode_args((user, past_episode)).unwrap(),
            )
            .expect("Failed to call deposit");

        let result: Result<(), PoolError> = decode_one(&deposit_result).unwrap();
        assert!(
            matches!(result, Err(PoolError::EpisodeNotActive)),
            "Expected EpisodeNotActive error for past episode, got: {:?}",
            result
        );
    }

    // Test deposit with non-stakable episode (should fail - pattern validation)
    let first_stakable_episode = get_stakable_episode(&pic, canister_id, 0);
    let non_stakable_episode = first_stakable_episode + 1;

    let subaccount_result = pic
        .query_call(
            canister_id,
            user,
            "get_deposit_subaccount",
            encode_args((user, non_stakable_episode)).unwrap(),
        )
        .expect("Failed to get deposit subaccount");
    let subaccount: [u8; 32] = decode_one(&subaccount_result).unwrap();

    let transfer_args = icp_canister_backend::TransferArg {
        from_subaccount: None,
        to: Account {
            owner: canister_id,
            subaccount: Some(subaccount.to_vec()),
        },
        amount: deposit_amount.clone(),
        fee: Some(TRANSFER_FEE.clone()),
        memo: None,
        created_at_time: None,
    };

    pic.update_call(
        ledger_id,
        user,
        "icrc1_transfer",
        encode_args((transfer_args,)).unwrap(),
    )
    .expect("Failed to transfer tokens");

    let deposit_result = pic
        .update_call(
            canister_id,
            user,
            "deposit",
            encode_args((user, non_stakable_episode)).unwrap(),
        )
        .expect("Failed to call deposit");

    let result: Result<(), PoolError> = decode_one(&deposit_result).unwrap();
    assert!(
        matches!(result, Err(PoolError::EpisodeNotActive)),
        "Expected EpisodeNotActive error for non-stakable episode, got: {:?}",
        result
    );

    // Test deposit in far future stakable episode (should fail - not yet active)
    let latest_stakable_episode = get_stakable_episode(&pic, canister_id, 7);
    let far_future_stakable_episode = latest_stakable_episode + 3;

    let subaccount_result = pic
        .query_call(
            canister_id,
            user,
            "get_deposit_subaccount",
            encode_args((user, far_future_stakable_episode)).unwrap(),
        )
        .expect("Failed to get deposit subaccount");
    let subaccount: [u8; 32] = decode_one(&subaccount_result).unwrap();

    let transfer_args = icp_canister_backend::TransferArg {
        from_subaccount: None,
        to: Account {
            owner: canister_id,
            subaccount: Some(subaccount.to_vec()),
        },
        amount: deposit_amount.clone(),
        fee: Some(TRANSFER_FEE.clone()),
        memo: None,
        created_at_time: None,
    };

    pic.update_call(
        ledger_id,
        user,
        "icrc1_transfer",
        encode_args((transfer_args,)).unwrap(),
    )
    .expect("Failed to transfer tokens");

    let deposit_result = pic
        .update_call(
            canister_id,
            user,
            "deposit",
            encode_args((user, far_future_stakable_episode)).unwrap(),
        )
        .expect("Failed to call deposit");

    let result: Result<(), PoolError> = decode_one(&deposit_result).unwrap();
    assert!(
        matches!(result, Err(PoolError::EpisodeNotActive)),
        "Expected EpisodeNotActive error for far future stakable episode, got: {:?}",
        result
    );

    // Test deposit in valid stakable episode (should succeed)
    let current_episode = get_stakable_episode(&pic, canister_id, 7); // Last stakable episode within range
    create_deposit(
        &pic,
        canister_id,
        ledger_id,
        user,
        deposit_amount.clone(),
        current_episode,
    );

    // Verify the deposit was created successfully
    let user_deposits = pic
        .query_call(
            canister_id,
            user,
            "get_user_deposits",
            encode_args((user,)).unwrap(),
        )
        .expect("Failed to get user deposits");
    let user_deposits: Vec<UserDepositInfo> = decode_one(&user_deposits).unwrap();

    assert_eq!(user_deposits.len(), 1, "User should have 1 deposit");
    assert_eq!(
        user_deposits[0].episode, current_episode,
        "Deposit should be in the stakable episode"
    );
}
#[test]
fn test_timer_episode_processing_exact_reduction() {
    let (pic, canister_id, ledger_id) = setup();
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let deposit_amount_1 = Nat::from(100_000_000u64);
    let deposit_amount_2 = Nat::from(200_000_000u64);

    let first_episode = get_stakable_episode(&pic, canister_id, 0);
    let second_episode = get_stakable_episode(&pic, canister_id, 1);

    // Create first deposit in first stakable episode
    create_deposit(
        &pic,
        canister_id,
        ledger_id,
        user,
        deposit_amount_1.clone(),
        first_episode,
    );

    // Record pool state after first deposit
    let pool_state_after_first = pic
        .query_call(
            canister_id,
            user,
            "get_pool_state",
            encode_args(()).unwrap(),
        )
        .expect("Failed to get pool state");
    let pool_after_first: PoolState = decode_one(&pool_state_after_first).unwrap();

    let expected_amount_1 = deposit_amount_1.clone() - TRANSFER_FEE.clone();
    assert_eq!(
        pool_after_first.total_assets, expected_amount_1,
        "Pool should have first deposit assets"
    );
    assert_eq!(
        pool_after_first.total_shares, expected_amount_1,
        "Pool should have first deposit shares"
    );

    // Create second deposit immediately in next stakable episode (before advancing time)
    create_deposit(
        &pic,
        canister_id,
        ledger_id,
        user,
        deposit_amount_2.clone(),
        second_episode,
    );

    // Record pool state after second deposit (both episodes should be active)
    let pool_state_after_second = pic
        .query_call(
            canister_id,
            user,
            "get_pool_state",
            encode_args(()).unwrap(),
        )
        .expect("Failed to get pool state");
    let pool_after_second: PoolState = decode_one(&pool_state_after_second).unwrap();

    let expected_amount_2 = deposit_amount_2.clone() - TRANSFER_FEE.clone();
    let expected_total_assets = expected_amount_1.clone() + expected_amount_2.clone();
    let expected_total_shares = expected_amount_1.clone() + expected_amount_2.clone();

    assert_eq!(
        pool_after_second.total_assets, expected_total_assets,
        "Pool should have both deposits assets"
    );
    assert_eq!(
        pool_after_second.total_shares, expected_total_shares,
        "Pool should have both deposits shares"
    );

    // Get episode data before processing
    let episode_1_before = pic
        .query_call(
            canister_id,
            user,
            "get_episode",
            encode_args((first_episode,)).unwrap(),
        )
        .expect("Failed to get episode 1");
    let episode_1_data: Option<icp_canister_backend::Episode> =
        decode_one(&episode_1_before).unwrap();
    assert!(episode_1_data.is_some(), "Episode 1 should exist");
    let episode_1 = episode_1_data.unwrap();

    assert_eq!(
        episode_1.episode_shares, expected_amount_1,
        "Episode 1 should have correct shares"
    );
    assert_eq!(
        episode_1.assets_staked, expected_amount_1,
        "Episode 1 should have correct assets"
    );

    // Advance time to make ONLY first stakable episode finish
    let first_episode_time_to_end = get_episode_time_to_end(&pic, first_episode);
    advance_time(&pic, first_episode_time_to_end);

    // Verify first episode was processed
    let episode_1_after = pic
        .query_call(
            canister_id,
            user,
            "get_episode",
            encode_args((first_episode,)).unwrap(),
        )
        .expect("Failed to get episode 1 after processing");
    let episode_1_processed: Option<icp_canister_backend::Episode> =
        decode_one(&episode_1_after).unwrap();
    assert!(
        episode_1_processed.is_some(),
        "Episode 1 should still exist after processing"
    );
    let _episode_1_final = episode_1_processed.unwrap();

    // Only first episode expired, second is still active
    let pool_state_final = pic
        .query_call(
            canister_id,
            user,
            "get_pool_state",
            encode_args(()).unwrap(),
        )
        .expect("Failed to get final pool state");
    let pool_final: PoolState = decode_one(&pool_state_final).unwrap();

    // Total should now be: (episode1 + episode2) - episode1 = episode2 only
    assert_eq!(
        pool_final.total_assets, expected_amount_2,
        "Pool assets should be reduced by exact episode 1 amount: {} - {} = {}",
        expected_total_assets, expected_amount_1, expected_amount_2
    );
    assert_eq!(
        pool_final.total_shares, expected_amount_2,
        "Pool shares should be reduced by exact episode 1 amount: {} - {} = {}",
        expected_total_shares, expected_amount_1, expected_amount_2
    );

    // Verify second episode is still active and unprocessed
    let episode_2_check = pic
        .query_call(
            canister_id,
            user,
            "get_episode",
            encode_args((second_episode,)).unwrap(),
        )
        .expect("Failed to get episode 2");
    let episode_2_data: Option<icp_canister_backend::Episode> =
        decode_one(&episode_2_check).unwrap();
    assert!(episode_2_data.is_some(), "Episode 2 should exist");
    let episode_2 = episode_2_data.unwrap();

    assert_eq!(
        episode_2.episode_shares, expected_amount_2,
        "Episode 2 should have correct shares"
    );
    assert_eq!(
        episode_2.assets_staked, expected_amount_2,
        "Episode 2 should have correct assets"
    );
}
#[test]
fn test_slash_function() {
    let (pic, canister_id, ledger_id) = setup();
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let executor = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap();
    let receiver = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();
    let deposit_amount_1 = Nat::from(300_000_000u64);
    let deposit_amount_2 = Nat::from(200_000_000u64);
    let slash_amount = Nat::from(100_000_000u64);

    let first_episode = get_stakable_episode(&pic, canister_id, 0);

    // Create two deposits
    create_deposit(
        &pic,
        canister_id,
        ledger_id,
        user,
        deposit_amount_1.clone(),
        first_episode,
    );

    let second_episode = get_stakable_episode(&pic, canister_id, 1);
    create_deposit(
        &pic,
        canister_id,
        ledger_id,
        user,
        deposit_amount_2.clone(),
        second_episode,
    );

    // Check user deposits before slash
    let deposits_before = pic
        .query_call(
            canister_id,
            user,
            "get_user_deposits",
            encode_args((user,)).unwrap(),
        )
        .expect("Failed to get user deposits");
    let user_deposits_before: Vec<UserDepositInfo> = decode_one(&deposits_before).unwrap();

    // Verify initial deposit values
    assert_eq!(user_deposits_before.len(), 2, "User should have 2 deposits");
    let expected_amount_1 = deposit_amount_1.clone() - TRANSFER_FEE.clone();
    let expected_amount_2 = deposit_amount_2.clone() - TRANSFER_FEE.clone();
    assert_eq!(
        user_deposits_before[0].amount, expected_amount_1,
        "First deposit should have correct initial amount"
    );
    assert_eq!(
        user_deposits_before[1].amount, expected_amount_2,
        "Second deposit should have correct initial amount"
    );

    // Check pool state before slash
    let pool_state_before = pic
        .query_call(
            canister_id,
            user,
            "get_pool_state",
            encode_args(()).unwrap(),
        )
        .expect("Failed to get pool state");
    let pool_before: PoolState = decode_one(&pool_state_before).unwrap();

    let total_assets_before = expected_amount_1.clone() + expected_amount_2.clone();
    assert_eq!(
        pool_before.total_assets, total_assets_before,
        "Pool should have correct total assets before slash"
    );

    // Execute slash
    let slash_result = pic
        .update_call(
            canister_id,
            executor,
            "slash",
            encode_args((receiver, slash_amount.clone())).unwrap(),
        )
        .expect("Failed to execute slash");
    let result: Result<(), PoolError> = decode_one(&slash_result).unwrap();
    assert!(
        matches!(result, Ok(_)),
        "Slash should succeed: {:?}",
        result
    );

    // Check user deposits after slash - values should be proportionally reduced
    let deposits_after = pic
        .query_call(
            canister_id,
            user,
            "get_user_deposits",
            encode_args((user,)).unwrap(),
        )
        .expect("Failed to get user deposits");
    let user_deposits_after: Vec<UserDepositInfo> = decode_one(&deposits_after).unwrap();

    // Calculate expected amounts based on proportional reduction
    let total_reduction = slash_amount.clone();
    let total_original = expected_amount_1.clone() + expected_amount_2.clone();

    // Calculate reduction for each deposit proportionally
    let reduction_1 = total_reduction.clone() * expected_amount_1.clone() / total_original.clone();
    let reduction_2 = total_reduction.clone() * expected_amount_2.clone() / total_original.clone();

    let expected_amount_after_1 = expected_amount_1.clone() - reduction_1;
    let expected_amount_after_2 = expected_amount_2.clone() - reduction_2;

    assert_eq!(
        user_deposits_after[0].amount, expected_amount_after_1,
        "First deposit should be reduced proportionally after slash"
    );
    assert_eq!(
        user_deposits_after[1].amount, expected_amount_after_2,
        "Second deposit should be reduced proportionally after slash"
    );

    // Check pool state after slash
    let pool_state_after = pic
        .query_call(
            canister_id,
            user,
            "get_pool_state",
            encode_args(()).unwrap(),
        )
        .expect("Failed to get pool state");
    let pool_after: PoolState = decode_one(&pool_state_after).unwrap();

    // Calculate actual reduction based on proportional slashing precision
    let reduction_1 =
        slash_amount.clone() * expected_amount_1.clone() / total_assets_before.clone();
    let reduction_2 =
        slash_amount.clone() * expected_amount_2.clone() / total_assets_before.clone();
    let actual_total_reduction = reduction_1.clone() + reduction_2.clone();
    let expected_assets = pool_before.total_assets.clone() - actual_total_reduction;
    assert_eq!(
        pool_after.total_assets, expected_assets,
        "Pool assets should be reduced by actual accumulated slash amount"
    );

    // Test withdrawal after slash - advance time first
    let first_episode_time_to_end = get_episode_time_to_end(&pic, first_episode);
    advance_time(&pic, first_episode_time_to_end);

    // Get user balance before withdrawal
    let user_account = Account {
        owner: user,
        subaccount: None,
    };
    let balance_before_withdraw = pic
        .query_call(
            ledger_id,
            user,
            "icrc1_balance_of",
            encode_args((user_account.clone(),)).unwrap(),
        )
        .expect("Failed to check user balance before withdrawal");
    let balance_before: Nat = decode_one(&balance_before_withdraw).unwrap();

    let withdraw_result = pic
        .update_call(canister_id, user, "withdraw", encode_args((0u64,)).unwrap())
        .expect("Failed to call withdraw");
    let withdraw_res: Result<(), PoolError> = decode_one(&withdraw_result).unwrap();
    assert!(
        matches!(withdraw_res, Ok(_)),
        "Withdrawal should succeed after slash"
    );

    // Check user balance after FIRST withdrawal only
    let balance_after_withdraw = pic
        .query_call(
            ledger_id,
            user,
            "icrc1_balance_of",
            encode_args((user_account.clone(),)).unwrap(),
        )
        .expect("Failed to check user balance after withdrawal");
    let balance_after: Nat = decode_one(&balance_after_withdraw).unwrap();

    // Calculate expected withdrawal amount (reduced by slash)
    let expected_withdrawal_amount = expected_amount_after_1.clone() - TRANSFER_FEE.clone();
    let expected_balance_after = balance_before.clone() + expected_withdrawal_amount.clone();

    assert_eq!(
        balance_after, expected_balance_after,
        "User should receive correct withdrawal amount after slash. Expected: {}, Got: {}",
        expected_balance_after, balance_after
    );

    // Verify that the second deposit is also possible to withdraw
    let second_episode_time_to_end = get_episode_time_to_end(&pic, second_episode);
    advance_time(&pic, second_episode_time_to_end);

    let withdraw_result = pic
        .update_call(canister_id, user, "withdraw", encode_args((1u64,)).unwrap())
        .expect("Failed to call withdraw");
    let withdraw_res_2: Result<(), PoolError> = decode_one(&withdraw_result).unwrap();
    assert!(
        matches!(withdraw_res_2, Ok(_)),
        "Second withdrawal should also succeed after slash"
    );

    // Verify receiver got the slashed tokens
    let receiver_account = Account {
        owner: receiver,
        subaccount: None,
    };
    let receiver_balance_result = pic
        .query_call(
            ledger_id,
            user,
            "icrc1_balance_of",
            encode_args((receiver_account,)).unwrap(),
        )
        .expect("Failed to check receiver balance");
    let receiver_balance: Nat = decode_one(&receiver_balance_result).unwrap();

    // Calculate actual accumulated slashed amount due to proportional precision
    let actual_accumulated_slashed = reduction_1.clone() + reduction_2.clone();
    let expected_received = actual_accumulated_slashed - TRANSFER_FEE.clone();
    assert_eq!(
        receiver_balance, expected_received,
        "Receiver should have received actual accumulated slashed tokens minus fees"
    );
}

#[test]
fn test_stakable_episode_functionality() {
    let (pic, canister_id, _) = setup();

    // Test that stakable episodes follow the pattern (episode % 3 == 2)
    for relative_episode in 0u8..8u8 {
        let stakable_episode = get_stakable_episode(&pic, canister_id, relative_episode);

        // Verify that the returned episode follows the stakable pattern
        assert_eq!(
            stakable_episode % 3,
            2,
            "Stakable episode {} should end in 2 when divided by 3",
            stakable_episode
        );
        println!(
            "Relative episode {} maps to absolute episode {}",
            relative_episode, stakable_episode
        );
    }

    // Test that relative episode 9 should fail (out of range)
    let panic_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        get_stakable_episode(&pic, canister_id, 9u8);
    }));
    assert!(
        panic_result.is_err(),
        "Expected panic for relative episode 9"
    );
}

#[test]
fn test_reward_rate_increase_decrease_during_episodes() {
    let (pic, canister_id, ledger_id) = setup();
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let reward_amount = Nat::from(365_000_000u64);

    // Check initial reward rate (should be 0)
    let initial_reward_rate_result = pic
        .query_call(
            canister_id,
            user,
            "get_pool_reward_rate",
            encode_args(()).unwrap(),
        )
        .expect("Failed to get initial pool reward rate");
    let initial_reward_rate: Nat = decode_one(&initial_reward_rate_result).unwrap();
    assert_eq!(
        initial_reward_rate,
        Nat::from(0u64),
        "Initial reward rate should be 0"
    );

    reward_pool(&pic, canister_id, ledger_id, user, reward_amount.clone())
        .expect("Reward pool should succeed");

    // Check reward rate after reward_pool (should be increased)
    let increased_reward_rate_result = pic
        .query_call(
            canister_id,
            user,
            "get_pool_reward_rate",
            encode_args(()).unwrap(),
        )
        .expect("Failed to get pool reward rate after reward_pool");
    let increased_reward_rate: Nat = decode_one(&increased_reward_rate_result).unwrap();
    assert!(
        increased_reward_rate > initial_reward_rate,
        "Reward rate should be increased after reward_pool. Initial: {}, After: {}",
        initial_reward_rate,
        increased_reward_rate
    );

    //  Calculate expected reward rate
    let current_time = pic.get_time().as_nanos_since_unix_epoch() / 1_000_000_000;
    let current_episode = current_time / icp_canister_backend::EPISODE_DURATION;
    let current_episode_finish_time =
        (current_episode + 1) * icp_canister_backend::EPISODE_DURATION;
    let reward_duration = 365 * 24 * 60 * 60 + (current_episode_finish_time - current_time);
    let actual_amount = reward_amount.clone();
    let expected_rate_increase = actual_amount.clone() / Nat::from(reward_duration);

    assert_eq!(
        increased_reward_rate, expected_rate_increase,
        "Reward rate should equal expected increase: {} tokens per second",
        expected_rate_increase
    );

    let last_reward_episode =
        (current_time + reward_duration) / icp_canister_backend::EPISODE_DURATION;
    let target_episode_for_decrease = last_reward_episode + 1;

    let time_to_reach_decrease_episode =
        (target_episode_for_decrease + 1) * icp_canister_backend::EPISODE_DURATION;
    let additional_time_needed = time_to_reach_decrease_episode - current_time;

    advance_time(&pic, additional_time_needed + 1);
    // Check reward rate after episode processing (should be decreased)
    let decreased_reward_rate_result = pic
        .query_call(
            canister_id,
            user,
            "get_pool_reward_rate",
            encode_args(()).unwrap(),
        )
        .expect("Failed to get pool reward rate after episode processing");
    let decreased_reward_rate: Nat = decode_one(&decreased_reward_rate_result).unwrap();

    // Should be exactly 0 since we decrease by the same amount we increased
    assert_eq!(
        decreased_reward_rate, Nat::from(0u64),
        "Reward rate should be back to 0 after processing episode with reward decrease. Final rate: {}",
        decreased_reward_rate
    );
}

#[test]
fn test_reward_distribution_middle_and_final() {
    let (pic, canister_id, ledger_id) = setup();
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let deposit_amount = Nat::from(100_000_000u64);
    let reward_amount = Nat::from(50_000_000u64);

    let stakable_episode = get_stakable_episode(&pic, canister_id, 0);
    create_deposit(&pic, canister_id, ledger_id, user, deposit_amount.clone(), stakable_episode);
    reward_pool(&pic, canister_id, ledger_id, user, reward_amount.clone())
        .expect("Reward pool should succeed");

    let earning_duration = get_episode_time_to_end(&pic, stakable_episode);

    //Middle  rewards distribution
    advance_time(&pic, earning_duration / 2);
    
    let middle_rewards = pic
        .query_call(canister_id, user, "get_deposits_rewards", encode_args((vec![0u64],)).unwrap())
        .map(|r| decode_one::<Nat>(&r).unwrap())
        .expect("Failed to get middle rewards");
    
    let expected_middle = Nat::from(earning_duration / 2);
    let allowed_error_middle = expected_middle.clone() / Nat::from(100u64);
    assert_with_error(&middle_rewards, &expected_middle, &allowed_error_middle, "Middle reward distribution");

    //  End  rewards distributed
    advance_time(&pic, earning_duration / 2);
    
    let final_rewards = pic
        .query_call(canister_id, user, "get_deposits_rewards", encode_args((vec![0u64],)).unwrap())
        .map(|r| decode_one::<Nat>(&r).unwrap())
        .expect("Failed to get final rewards");
    
    let expected_final = Nat::from(earning_duration);
    let allowed_error_final = expected_final.clone() / Nat::from(100u64);
    assert_with_error(&final_rewards, &expected_final, &allowed_error_final, "Final reward distribution");
}

#[test]
fn test_reward_distribution_middle_and_final1() {
    let (pic, canister_id, ledger_id) = setup();
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    

    let deposit_amount = Nat::from(100_000_000u64); // 1 ckBTC (8 decimals)
    let reward_amount = Nat::from(50_000_000u64);   // 0.5 ckBTC

    let stakable_episode = get_stakable_episode(&pic, canister_id, 0);
    create_deposit(&pic, canister_id, ledger_id, user, deposit_amount.clone(), stakable_episode);
    reward_pool(&pic, canister_id, ledger_id, user, reward_amount.clone())
        .expect("Reward pool should succeed");

    
    let reward_timeline = icp_canister_backend::EPISODE_DURATION * 12;

    // Middle rewards distribution
    advance_time(&pic, reward_timeline / 2);
    
    let middle_rewards = pic
        .query_call(canister_id, user, "get_deposits_rewards", encode_args((vec![0u64],)).unwrap())
        .map(|r| decode_one::<Nat>(&r).unwrap())
        .expect("Failed to get middle rewards");
    
    //  Final rewards should equal all rewards (reward_amount)
    let expected_middle = reward_amount.clone() / Nat::from(2u64);
    
    //  0.1 cent error tolerance in ckBTC
    const ALLOWED_ERROR: u64 = 1000; // 0.1 cent in ckBTC 
    let allowed_error_middle = Nat::from(ALLOWED_ERROR);
    assert_with_error(&middle_rewards, &expected_middle, &allowed_error_middle, "Middle reward distribution");
// End rewards distributed
    advance_time(&pic, reward_timeline / 2);
    
    let final_rewards = pic
        .query_call(canister_id, user, "get_deposits_rewards", encode_args((vec![0u64],)).unwrap())
        .map(|r| decode_one::<Nat>(&r).unwrap())
        .expect("Failed to get final rewards");
    
    // final distributed rewards should be equal all of the rewards"
    let expected_final = reward_amount.clone();
    let allowed_error_final = Nat::from(ALLOWED_ERROR);
    assert_with_error(&final_rewards, &expected_final, &allowed_error_final, "Final reward distribution");
}



#[test]
fn test_full_reward_distribution() {
    let (pic, canister_id, ledger_id) = setup();
    let user = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let deposit_amount = Nat::from(100_000_000u64);
    let reward_amount = Nat::from(50_000_000u64);
    const ALLOWED_ERROR: u64 = 1000;
    let stakable_episode = get_stakable_episode(&pic, canister_id, 0);
    create_deposit(&pic, canister_id, ledger_id, user, deposit_amount.clone(), stakable_episode);
    reward_pool(&pic, canister_id, ledger_id, user, reward_amount.clone())
        .expect("Reward pool should succeed");

    // Advance time to the end of reward distribution time (1 year + episode)
    let one_year = 365 * 24 * 60 * 60;
    let current_episode = get_current_episode(&pic, canister_id);
    let episode_end_time = get_episode_time_to_end(&pic, current_episode);
    advance_time(&pic, one_year + episode_end_time);
    
    let final_rewards = pic
        .query_call(canister_id, user, "get_deposits_rewards", encode_args((vec![0u64],)).unwrap())
        .map(|r| decode_one::<Nat>(&r).unwrap())
        .expect("Failed to get final rewards");
    
    // Check that it is equal to amount rewarded initially
    let expected_final = reward_amount.clone();
    let allowed_error_final = Nat::from(ALLOWED_ERROR);
    assert_with_error(&final_rewards, &expected_final, &allowed_error_final, "Full reward distribution");
}