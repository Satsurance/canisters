#!/bin/bash

# Add rewards to the canister by calling reward_pool() function
# This transfers tokens from reward subaccount to main canister and increases reward rate
# Requires .env.local with LEDGER, BACKEND, NETWORK variables

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

info() { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; }

ROOT_DIR=$(cd "$(dirname "$0")" && pwd)
cd "$ROOT_DIR"

# Load env
if [ -f .env.local ]; then
  # shellcheck disable=SC1091
  source .env.local
else
  error ".env.local not found. Run ./deploy.sh first."
  exit 1
fi

: "${LEDGER:?LEDGER not set in .env.local}"
: "${BACKEND:?BACKEND not set in .env.local}"
: "${NETWORK:=local}"

# Get amount from argument or default to 1 token
AMOUNT_E8S=${1:-100000000}
info "Adding rewards amount (base units): $AMOUNT_E8S"

# Ensure replica for local
if [ "${NETWORK}" = "local" ]; then
  if ! curl -s http://127.0.0.1:4943/api/v2/status >/dev/null 2>&1; then
    info "Starting local replica..."
    dfx start --background
    sleep 1
  fi
fi

# 1) Get reward subaccount
info "Getting reward subaccount..."
REWARD_SUB_RAW=$(dfx canister call "$BACKEND" get_reward_subaccount --network "$NETWORK")
REWARD_SUB_CLEAN=$(echo "$REWARD_SUB_RAW" | sed 's/,$//')
info "Reward subaccount: $REWARD_SUB_CLEAN"

# 2) Transfer tokens to reward subaccount
info "Transferring $AMOUNT_E8S tokens to reward subaccount..."
TRANSFER_RES=$(dfx canister call "$LEDGER" icrc1_transfer "(record { to = record { owner = principal \"$BACKEND\"; subaccount = opt $REWARD_SUB_CLEAN }; amount = $AMOUNT_E8S:nat; fee = null; memo = null; created_at_time = null })" --network "$NETWORK")
info "Transfer result: $TRANSFER_RES"

if [[ "$TRANSFER_RES" == *"Err"* ]]; then
  error "Transfer to reward subaccount failed: $TRANSFER_RES"
  exit 1
fi

# 3) Call reward_pool() to process the rewards
info "Calling reward_pool() to process rewards..."
REWARD_RES=$(dfx canister call "$BACKEND" reward_pool --network "$NETWORK")
info "Reward pool result: $REWARD_RES"

if [[ "$REWARD_RES" == *"Err"* ]]; then
  error "Reward pool call failed: $REWARD_RES"
  exit 1
fi

# 4) Check updated reward rate
info "Checking updated reward rate..."
REWARD_RATE=$(dfx canister call "$BACKEND" get_pool_reward_rate --network "$NETWORK")
info "Current reward rate: $REWARD_RATE"

# 5) Check pool state
info "Checking pool state..."
POOL_STATE=$(dfx canister call "$BACKEND" get_pool_state --network "$NETWORK")
info "Pool state: $POOL_STATE"

success "Rewards added successfully!"

