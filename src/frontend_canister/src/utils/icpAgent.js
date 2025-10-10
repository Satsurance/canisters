import { HttpAgent, Actor } from '@dfinity/agent';

// Backend canister IDL
export const backendIdlFactory = ({ IDL }) => {
  const PoolError = IDL.Variant({
    NoDeposit: IDL.Null,
    InsufficientBalance: IDL.Null,
    TransferFailed: IDL.Null,
    LedgerCallFailed: IDL.Null,
    LedgerNotSet: IDL.Null,
    NotOwner: IDL.Null,
    TimelockNotExpired: IDL.Null,
    EpisodeNotActive: IDL.Null,
    NotSlashingExecutor: IDL.Null,
    EpisodeNotStakable: IDL.Null,
    NotPoolManager: IDL.Null,
    ProductNotActive: IDL.Null,
    CoverageDurationTooLong: IDL.Null,
    CoverageDurationTooShort: IDL.Null,
    NotEnoughAssetsToCover: IDL.Null,
    ProductNotFound: IDL.Null,
    InvalidProductParameters: IDL.Null,
  });

  const UserDepositInfo = IDL.Record({
    deposit_id: IDL.Nat64,
    episode: IDL.Nat64,
    shares: IDL.Nat,
    amount: IDL.Nat,
  });

  const Product = IDL.Record({
    name: IDL.Text,
    product_id: IDL.Nat64,
    annual_percent: IDL.Nat64,
    max_coverage_duration: IDL.Nat64,
    max_pool_allocation_percent: IDL.Nat64,
    allocation: IDL.Nat,
    last_allocation_update: IDL.Nat64,
    active: IDL.Bool,
  });

  const Coverage = IDL.Record({
    coverage_id: IDL.Nat64,
    buyer: IDL.Principal,
    covered_account: IDL.Principal,
    product_id: IDL.Nat64,
    coverage_amount: IDL.Nat,
    premium_amount: IDL.Nat,
    start_time: IDL.Nat64,
    end_time: IDL.Nat64,
  });

  return IDL.Service({
    get_current_episode_id: IDL.Func([], [IDL.Nat64], ['query']),
    get_pool_state: IDL.Func([], [IDL.Record({ total_assets: IDL.Nat, total_shares: IDL.Nat })], ['query']),
    get_pool_reward_rate: IDL.Func([], [IDL.Nat], ['query']),
    get_user_deposits: IDL.Func([IDL.Principal], [IDL.Vec(UserDepositInfo)], ['query']),
    get_deposits_rewards: IDL.Func([IDL.Vec(IDL.Nat64)], [IDL.Nat], ['query']),
    withdraw_rewards: IDL.Func([IDL.Vec(IDL.Nat64)], [IDL.Variant({ Ok: IDL.Nat, Err: PoolError })], []),
    // ICP staking flow
    get_deposit_subaccount: IDL.Func([IDL.Principal, IDL.Nat64], [IDL.Vec(IDL.Nat8)], ['query']),
    deposit: IDL.Func([IDL.Principal, IDL.Nat64], [IDL.Variant({ Ok: IDL.Null, Err: PoolError })], []),
    // Coverage flow
    get_purchase_subaccount: IDL.Func([IDL.Principal, IDL.Nat64], [IDL.Vec(IDL.Nat8)], ['query']),
    get_products: IDL.Func([], [IDL.Vec(Product)], ['query']),
    get_coverages: IDL.Func([IDL.Principal], [IDL.Vec(Coverage)], ['query']),
    purchase_coverage: IDL.Func(
      [IDL.Nat64, IDL.Principal, IDL.Nat64, IDL.Nat],
      [IDL.Variant({ Ok: IDL.Null, Err: PoolError })],
      []
    ),
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
  if (host.includes('127.0.0.1') || host.includes('localhost')) {
    try { await agent.fetchRootKey(); } catch (_) { }
  }
  return Actor.createActor(backendIdlFactory, { agent, canisterId });
}

export function createBackendActorWithAgent(agent, canisterId) {
  return Actor.createActor(backendIdlFactory, { agent, canisterId });
}

export function createLedgerActorWithAgent(agent, canisterId) {
  return Actor.createActor(ledgerIdlFactory, { agent, canisterId });
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
