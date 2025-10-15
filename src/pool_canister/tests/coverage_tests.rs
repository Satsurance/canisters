use candid::{Nat, Principal};
use commons::{
    calculate_premium, create_deposit, get_stakable_episode_with_client, purchase_coverage,
    LedgerCanisterClient, PoolCanisterClient,
};
use commons::clients::ledger::TransferResult;

mod setup;
use setup::setup;

#[test]
fn test_create_product_and_purchase_coverage() {
    let (pic, pool_canister, ledger_id) = setup();
    let mut pool_client = PoolCanisterClient::new(&pic, pool_canister);
    let mut ledger_client = LedgerCanisterClient::new(&pic, ledger_id);

    let user1 = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap();
    let buyer = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let pool_manager = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();
    // Create deposit
    let current_episode = get_stakable_episode_with_client(&pool_client, 3);
    let deposit_amount = Nat::from(1_000_000_000u64);

    create_deposit(
        &mut pool_client,
        &mut ledger_client,
        user1,
        deposit_amount.clone(),
        current_episode,
    )
    .expect("Deposit should succeed");

    // Create product
    let product_name = "Bridge Insurance".to_string();
    let annual_percent = 500u64; // 5%
    let max_coverage_duration = pool_canister::EPISODE_DURATION * 6; // 6 episodes
    let max_pool_allocation_percent = 5000u64; // 50%

    let product_id = pool_client.connect(pool_manager).create_product(
        product_name,
        annual_percent,
        max_coverage_duration,
        max_pool_allocation_percent,
    );

    assert!(product_id.is_ok(), "Product creation should succeed");
    let product_id = product_id.unwrap();

    // Verify product was created
    let all_products = pool_client.get_products();
    assert!(!all_products.is_empty(), "Should have at least one product");

    let product = all_products
        .iter()
        .find(|p| p.product_id == product_id)
        .expect("Created product should be in the list");

    assert_eq!(product.annual_percent, annual_percent);
    assert_eq!(product.max_coverage_duration, max_coverage_duration);
    assert_eq!(product.active, true);

    // Purchase coverage
    let covered_account = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();
    let coverage_duration = pool_canister::EPISODE_DURATION * 3; // 3 episodes
    let coverage_amount = Nat::from(100_000_000u64); // 1 BTC

    // Calculate premium
    let premium_amount =
        calculate_premium(coverage_duration, annual_percent, coverage_amount.clone());

    // Purchase coverage using helper function
    let result = purchase_coverage(
        &mut pool_client,
        &mut ledger_client,
        buyer,
        product_id,
        covered_account,
        coverage_duration,
        coverage_amount.clone(),
        premium_amount.clone(),
    );

    assert!(
        result.is_ok(),
        "Coverage purchase should succeed: {:?}",
        result
    );

    // Verify product allocation increased
    let all_products_after = pool_client.get_products();
    let product_after = all_products_after
        .iter()
        .find(|p| p.product_id == product_id)
        .expect("Product should still exist after purchase");

    assert_eq!(
        product_after.allocation, coverage_amount,
        "Product allocation should equal coverage amount"
    );

    // Verify total cover allocation
    let total_cover = pool_client.get_total_cover_allocation();
    assert_eq!(
        total_cover, coverage_amount,
        "Total cover allocation should equal coverage amount"
    );

    // Verify coverage is stored and retrievable by buyer
    let buyer_coverages = pool_client.get_coverages(buyer);
    assert_eq!(
        buyer_coverages.len(),
        1,
        "Buyer should have exactly 1 coverage"
    );

    let stored_coverage = &buyer_coverages[0];
    assert_eq!(
        stored_coverage.coverage_id, 0,
        "First coverage should have ID 0"
    );
    assert_eq!(stored_coverage.buyer, buyer, "Coverage buyer should match");
    assert_eq!(
        stored_coverage.covered_account, covered_account,
        "Covered account should match"
    );
    assert_eq!(
        stored_coverage.product_id, product_id,
        "Product ID should match"
    );
    assert_eq!(
        stored_coverage.coverage_amount, coverage_amount,
        "Coverage amount should match"
    );
    assert_eq!(
        stored_coverage.premium_amount, premium_amount,
        "Premium amount should match"
    );
    assert_eq!(
        stored_coverage.end_time - stored_coverage.start_time,
        coverage_duration,
        "Duration should match coverage_duration"
    );
}

#[test]
fn test_coverage_allocation_limits() {
    let (pic, pool_canister, ledger_id) = setup();
    let mut pool_client = PoolCanisterClient::new(&pic, pool_canister);
    let mut ledger_client = LedgerCanisterClient::new(&pic, ledger_id);

    let user1 = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap();
    let buyer = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let pool_manager = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();
    // Create deposit
    let current_episode = get_stakable_episode_with_client(&pool_client, 2);
    let deposit_amount = Nat::from(1_000_000_000u64);

    create_deposit(
        &mut pool_client,
        &mut ledger_client,
        user1,
        deposit_amount.clone(),
        current_episode,
    )
    .expect("Deposit should succeed");

    // Create product with 50% max allocation
    let product_id = pool_client
        .connect(pool_manager)
        .create_product(
            "Test Product".to_string(),
            500u64,
            pool_canister::EPISODE_DURATION * 6,
            5000u64,
        )
        .unwrap();

    // Try to purchase coverage exceeding allocation limit
    let covered_account = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();
    let coverage_duration = pool_canister::EPISODE_DURATION * 3;
    let coverage_amount = Nat::from(600_000_000u64); // 60% of pool (exceeds 50% limit)

    // Calculate premium
    let premium_amount = calculate_premium(coverage_duration, 500u64, coverage_amount.clone());

    // Try to purchase - should fail due to allocation limit (checked before payment)
    let result = purchase_coverage(
        &mut pool_client,
        &mut ledger_client,
        buyer,
        product_id,
        covered_account,
        coverage_duration,
        coverage_amount,
        premium_amount,
    );
    assert!(
        result.is_err(),
        "Should fail due to allocation limit: {:?}",
        result
    );
}

#[test]
fn test_independent_product_allocations() {
    let (pic, pool_canister, ledger_id) = setup();
    let mut pool_client = PoolCanisterClient::new(&pic, pool_canister);
    let mut ledger_client = LedgerCanisterClient::new(&pic, ledger_id);

    let user1 = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap();
    let buyer = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let pool_manager = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();
    // Create deposit
    let current_episode = get_stakable_episode_with_client(&pool_client, 2);
    let deposit_amount = Nat::from(1_000_000_000u64);

    create_deposit(
        &mut pool_client,
        &mut ledger_client,
        user1,
        deposit_amount.clone(),
        current_episode,
    )
    .expect("Deposit should succeed");

    // Create two products
    let product1_id = pool_client
        .connect(pool_manager)
        .create_product(
            "Bridge Insurance".to_string(),
            500u64,
            pool_canister::EPISODE_DURATION * 6,
            5000u64,
        )
        .unwrap();

    let product2_id = pool_client
        .connect(pool_manager)
        .create_product(
            "DeFi Insurance".to_string(),
            800u64,
            pool_canister::EPISODE_DURATION * 6,
            10000u64,
        )
        .unwrap();

    let covered_account = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();
    let coverage_duration = pool_canister::EPISODE_DURATION * 3;

    // Purchase 250M from product1 (25% - within 50% limit)
    let coverage_amount1 = Nat::from(250_000_000u64);
    let premium_amount1 = calculate_premium(coverage_duration, 500u64, coverage_amount1.clone());
    let result1 = purchase_coverage(
        &mut pool_client,
        &mut ledger_client,
        buyer,
        product1_id,
        covered_account,
        coverage_duration,
        coverage_amount1,
        premium_amount1,
    );
    assert!(
        result1.is_ok(),
        "First coverage should succeed: {:?}",
        result1
    );

    // Purchase 500M from product2 (50% - within 100% limit)
    let coverage_amount2 = Nat::from(500_000_000u64);
    let premium_amount2 = calculate_premium(coverage_duration, 800u64, coverage_amount2.clone());
    let result2 = purchase_coverage(
        &mut pool_client,
        &mut ledger_client,
        buyer,
        product2_id,
        covered_account,
        coverage_duration,
        coverage_amount2,
        premium_amount2,
    );
    assert!(
        result2.is_ok(),
        "Second coverage should succeed: {:?}",
        result2
    );

    // Total coverage sold: 750M (75% of pool)
    let total_cover = pool_client.get_total_cover_allocation();

    assert_eq!(
        total_cover,
        Nat::from(750_000_000u64),
        "Total allocation should be 750M (75% of pool)"
    );
}

#[test]
fn test_excess_balance_refund() {
    let (pic, pool_canister, ledger_id) = setup();
    let mut pool_client = PoolCanisterClient::new(&pic, pool_canister);
    let mut ledger_client = LedgerCanisterClient::new(&pic, ledger_id);

    let user1 = Principal::from_text("rdmx6-jaaaa-aaaaa-aaadq-cai").unwrap();
    let buyer = Principal::from_text("xkbqi-2qaaa-aaaah-qbpqq-cai").unwrap();
    let pool_manager = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();

    // Create deposit
    let current_episode = get_stakable_episode_with_client(&pool_client, 2);
    let deposit_amount = Nat::from(1_000_000_000u64);

    create_deposit(
        &mut pool_client,
        &mut ledger_client,
        user1,
        deposit_amount.clone(),
        current_episode,
    )
    .expect("Deposit should succeed");

    // Create product
    let product_id = pool_client
        .connect(pool_manager)
        .create_product(
            "Test Product".to_string(),
            500u64,
            pool_canister::EPISODE_DURATION * 6,
            5000u64,
        )
        .unwrap();

    // Get buyer's initial balance
    let buyer_initial_balance = ledger_client.icrc1_balance_of(pool_canister::Account {
        owner: buyer,
        subaccount: None,
    });

    // Purchase coverage with excess payment
    let covered_account = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();
    let coverage_duration = pool_canister::EPISODE_DURATION * 3;
    let coverage_amount = Nat::from(100_000_000u64);

    // Calculate actual premium needed
    let premium_amount = calculate_premium(coverage_duration, 500u64, coverage_amount.clone());

    // Transfer MORE than needed to the purchase subaccount
    let excess_amount = Nat::from(50_000u64); // Extra 50k units
    let total_payment = premium_amount.clone() + excess_amount.clone();

    let subaccount = pool_client
        .connect(buyer)
        .get_purchase_subaccount(buyer, product_id);

    let transfer_args = pool_canister::TransferArg {
        from_subaccount: None,
        to: pool_canister::Account {
            owner: pool_canister,
            subaccount: Some(subaccount.to_vec()),
        },
        amount: total_payment.clone(),
        fee: Some(Nat::from(10u64)),
        memo: None,
        created_at_time: None,
    };

    // Transfer excess amount to purchase subaccount
    let transfer_result = ledger_client.connect(buyer).icrc1_transfer(transfer_args);
    assert!(
        matches!(transfer_result, TransferResult::Ok(_)),
        "Transfer to subaccount should succeed"
    );

    // Purchase coverage - should use only premium_amount and refund excess
    let result = pool_client.purchase_coverage(
        product_id,
        covered_account,
        coverage_duration,
        coverage_amount.clone(),
    );

    assert!(
        result.is_ok(),
        "Coverage purchase should succeed: {:?}",
        result
    );

    // Verify buyer received refund (checking final balance)
    let buyer_final_balance = ledger_client.icrc1_balance_of(pool_canister::Account {
        owner: buyer,
        subaccount: None,
    });

    // Expected: initial - total_payment - transfer_fee + excess_refund - transfer_fee_for_refund
    // = initial - total_payment - 10 + excess - 10
    // = initial - premium - excess - 10 + excess - 10
    // = initial - premium - 20
    let expected_final_balance = buyer_initial_balance - premium_amount.clone() - Nat::from(20u64);

    assert_eq!(
        buyer_final_balance, expected_final_balance,
        "Buyer should receive refund of excess amount minus fees"
    );

    // Verify coverage was created successfully
    let buyer_coverages = pool_client.get_coverages(buyer);
    assert_eq!(
        buyer_coverages.len(),
        1,
        "Buyer should have exactly 1 coverage"
    );

    let stored_coverage = &buyer_coverages[0];
    assert_eq!(
        stored_coverage.premium_amount, premium_amount,
        "Premium amount should match calculated amount (not the excess payment)"
    );
}
