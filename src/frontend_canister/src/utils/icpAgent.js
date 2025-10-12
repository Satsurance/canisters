import { HttpAgent, Actor } from '@dfinity/agent';
import { handlePlugError } from './errorHandler.js';

// Backend canister IDL
export const backendIdlFactory = ({ IDL }) => {
  const PoolError = IDL.Variant({
    NotEnoughAssetsToCover: IDL.Null,
    NotSlashingExecutor: IDL.Null,
    InvalidProductParameters: IDL.Null,
    NoDeposit: IDL.Null,
    CoverageDurationTooShort: IDL.Null,
    InsufficientBalance: IDL.Null,
    LedgerCallFailed: IDL.Null,
    ProductNotFound: IDL.Null,
    ProductNotActive: IDL.Null,
    NotPoolManager: IDL.Null,
    NotOwner: IDL.Null,
    EpisodeNotStakable: IDL.Null,
    LedgerNotSet: IDL.Null,
    TransferFailed: IDL.Null,
    CoverageDurationTooLong: IDL.Null,
    TimelockNotExpired: IDL.Null,
    EpisodeNotActive: IDL.Null,
  });

  const Coverage = IDL.Record({
    coverage_amount: IDL.Nat,
    product_id: IDL.Nat64,
    end_time: IDL.Nat64,
    start_time: IDL.Nat64,
    covered_account: IDL.Principal,
    buyer: IDL.Principal,
    coverage_id: IDL.Nat64,
    premium_amount: IDL.Nat,
  });

  const Deposit = IDL.Record({
    shares: IDL.Nat,
    reward_per_share: IDL.Nat,
    rewards_collected: IDL.Nat,
    episode: IDL.Nat64,
  });

  const Episode = IDL.Record({
    episode_shares: IDL.Nat,
    assets_staked: IDL.Nat,
    reward_decrease: IDL.Nat,
    coverage_decrease: IDL.Nat,
    acc_reward_per_share_on_expire: IDL.Nat,
  });

  const Product = IDL.Record({
    active: IDL.Bool,
    product_id: IDL.Nat64,
    name: IDL.Text,
    max_pool_allocation_percent: IDL.Nat64,
    annual_percent: IDL.Nat64,
    max_coverage_duration: IDL.Nat64,
    allocation: IDL.Nat,
    last_allocation_update: IDL.Nat64,
  });

  const PoolState = IDL.Record({
    total_shares: IDL.Nat,
    total_assets: IDL.Nat,
  });

  const UserDepositInfo = IDL.Record({
    deposit_id: IDL.Nat64,
    episode: IDL.Nat64,
    shares: IDL.Nat,
    amount: IDL.Nat,
  });

  const Result = IDL.Variant({ Ok: IDL.Nat64, Err: PoolError });
  const Result_1 = IDL.Variant({ Ok: IDL.Null, Err: PoolError });
  const Result_2 = IDL.Variant({ Ok: IDL.Nat, Err: PoolError });

  return IDL.Service({
    // Product management
    create_product: IDL.Func([IDL.Text, IDL.Nat64, IDL.Nat64, IDL.Nat64], [Result], []),
    set_product: IDL.Func([IDL.Nat64, IDL.Nat64, IDL.Nat64, IDL.Nat64, IDL.Bool], [Result_1], []),
    get_products: IDL.Func([], [IDL.Vec(Product)], ['query']),

    // Coverage purchase and queries
    purchase_coverage: IDL.Func([IDL.Nat64, IDL.Principal, IDL.Nat64, IDL.Nat], [Result_1], []),
    get_coverage: IDL.Func([IDL.Nat64], [IDL.Opt(Coverage)], ['query']),
    get_coverages: IDL.Func([IDL.Principal], [IDL.Vec(Coverage)], ['query']),
    get_purchase_subaccount: IDL.Func([IDL.Principal, IDL.Nat64], [IDL.Vec(IDL.Nat8)], ['query']),
    get_total_cover_allocation: IDL.Func([], [IDL.Nat], ['query']),

    // Deposit/Staking management
    deposit: IDL.Func([IDL.Principal, IDL.Nat64], [Result_1], []),
    withdraw: IDL.Func([IDL.Nat64], [Result_1], []),
    get_deposit_subaccount: IDL.Func([IDL.Principal, IDL.Nat64], [IDL.Vec(IDL.Nat8)], ['query']),
    get_deposit: IDL.Func([IDL.Nat64], [IDL.Opt(Deposit)], ['query']),
    get_user_deposits: IDL.Func([IDL.Principal], [IDL.Vec(UserDepositInfo)], ['query']),

    // Rewards management
    get_deposits_rewards: IDL.Func([IDL.Vec(IDL.Nat64)], [IDL.Nat], ['query']),
    withdraw_rewards: IDL.Func([IDL.Vec(IDL.Nat64)], [Result_2], []),
    reward_pool: IDL.Func([], [Result_1], []),
    get_reward_subaccount: IDL.Func([], [IDL.Vec(IDL.Nat8)], ['query']),
    get_pool_reward_rate: IDL.Func([], [IDL.Nat], ['query']),

    // Pool state queries
    get_current_episode_id: IDL.Func([], [IDL.Nat64], ['query']),
    get_pool_state: IDL.Func([], [PoolState], ['query']),
    get_episode: IDL.Func([IDL.Nat64], [IDL.Opt(Episode)], ['query']),
    update_episodes_state: IDL.Func([], [], []),

    // Admin functions
    get_executor_principal: IDL.Func([], [IDL.Principal], ['query']),
    get_pool_manager_principal: IDL.Func([], [IDL.Principal], ['query']),
    set_executor_principal: IDL.Func([IDL.Principal], [Result_1], []),
    set_pool_manager_principal: IDL.Func([IDL.Principal], [Result_1], []),
    slash: IDL.Func([IDL.Principal, IDL.Nat], [Result_1], []),
  });
};

// Claim Canister IDL
export const claimIdlFactory = ({ IDL }) => {
  const ClaimStatus = IDL.Variant({
    Pending: IDL.Null,
    Approved: IDL.Null,
    Executing: IDL.Null,
    Executed: IDL.Null,
  });
  const ClaimInfo = IDL.Record({
    id: IDL.Nat64,
    receiver: IDL.Principal,
    amount: IDL.Nat,
    pool_canister_id: IDL.Principal,
    description: IDL.Text,
    status: ClaimStatus,
    created_at: IDL.Nat64,
    approved_at: IDL.Opt(IDL.Nat64),
    approved_by: IDL.Opt(IDL.Principal),
  });
  const ClaimError = IDL.Variant({
    NotFound: IDL.Null,
    AlreadyApproved: IDL.Null,
    AlreadyExecuted: IDL.Null,
    NotApprover: IDL.Null,
    NotApproved: IDL.Null,
    TimelockNotExpired: IDL.Null,
    PoolCallFailed: IDL.Text,
    InvalidStatus: IDL.Null,
    InsufficientPermissions: IDL.Null,
  });
  return IDL.Service({
    add_claim: IDL.Func([IDL.Principal, IDL.Nat, IDL.Principal, IDL.Text], [IDL.Variant({ Ok: IDL.Nat64, Err: ClaimError })], []),
    approve_claim: IDL.Func([IDL.Nat64], [IDL.Variant({ Ok: IDL.Null, Err: ClaimError })], []),
    execute_claim: IDL.Func([IDL.Nat64], [IDL.Variant({ Ok: IDL.Null, Err: ClaimError })], []),
    get_claim: IDL.Func([IDL.Nat64], [IDL.Opt(ClaimInfo)], ['query']),
    is_approver: IDL.Func([IDL.Principal], [IDL.Bool], ['query']),
    add_approver: IDL.Func([IDL.Principal], [IDL.Variant({ Ok: IDL.Null, Err: ClaimError })], []),
    remove_approver: IDL.Func([IDL.Principal], [IDL.Variant({ Ok: IDL.Null, Err: ClaimError })], []),
    get_owner: IDL.Func([], [IDL.Principal], ['query']),
  });
};

// ICRC-1 Ledger IDL
export const ledgerIdlFactory = ({ IDL }) => {
  const Account = IDL.Record({ owner: IDL.Principal, subaccount: IDL.Opt(IDL.Vec(IDL.Nat8)) });
  const TransferArg = IDL.Record({
    from_subaccount: IDL.Opt(IDL.Vec(IDL.Nat8)),
    to: Account,
    amount: IDL.Nat,
    fee: IDL.Opt(IDL.Nat),
    memo: IDL.Opt(IDL.Vec(IDL.Nat8)),
    created_at_time: IDL.Opt(IDL.Nat64),
  });
  const TransferError = IDL.Variant({
    BadFee: IDL.Record({ expected_fee: IDL.Nat }),
    BadBurn: IDL.Record({ min_burn_amount: IDL.Nat }),
    InsufficientFunds: IDL.Record({ balance: IDL.Nat }),
    TooOld: IDL.Null,
    CreatedInFuture: IDL.Record({ ledger_time: IDL.Nat64 }),
    TemporarilyUnavailable: IDL.Null,
    Duplicate: IDL.Record({ duplicate_of: IDL.Nat }),
    GenericError: IDL.Record({ error_code: IDL.Nat, message: IDL.Text }),
  });
  return IDL.Service({
    icrc1_transfer: IDL.Func([TransferArg], [IDL.Variant({ Ok: IDL.Nat, Err: TransferError })], []),
    icrc1_balance_of: IDL.Func([Account], [IDL.Nat], ['query']),
    icrc1_decimals: IDL.Func([], [IDL.Nat8], ['query']),
    icrc1_fee: IDL.Func([], [IDL.Nat], ['query']),
  });
};

export async function createBackendActor(canisterId, host) {
  const agent = new HttpAgent({
    host,
    verifyQuerySignatures: false
  });
  return Actor.createActor(backendIdlFactory, { agent, canisterId });
}


// Create actors via Plug (preferred for signing update calls)
export async function createBackendActorWithPlug(canisterId) {
  if (!window.ic?.plug) throw new Error('Plug not available');
  return window.ic.plug.createActor({ canisterId, interfaceFactory: backendIdlFactory });
}

export async function createLedgerActorWithPlug(canisterId) {
  if (!window.ic?.plug) throw new Error('Plug not available');
  console.log('window.ic.plug', window.ic.plug);
  return window.ic.plug.createActor({ canisterId, interfaceFactory: ledgerIdlFactory });
}

export async function createClaimActorWithPlug(canisterId) {
  if (!window.ic?.plug) throw new Error('Plug not available');
  return window.ic.plug.createActor({ canisterId, interfaceFactory: claimIdlFactory });
}