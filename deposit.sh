#!/bin/bash

# Create a deposit for a user by transferring 1 token to their deposit subaccount
# Then call backend deposit(user, current_episode)
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

# Get principal from argument or prompt
USER_PRINCIPAL=${1:-}
if [ -z "$USER_PRINCIPAL" ]; then
  read -rp "Enter user principal (e.g., $(dfx identity get-principal 2>/dev/null || echo 'aaaaa-aa')): " USER_PRINCIPAL
fi

# Amount in smallest units (default 1 token = 10^8 for ICRC-1 standard)
AMOUNT_E8S=${2:-100000000}

info "Using principal: $USER_PRINCIPAL"
info "Using amount (base units): $AMOUNT_E8S"

# Ensure replica for local
if [ "${NETWORK}" = "local" ]; then
  if ! curl -s http://127.0.0.1:4943/api/v2/status >/dev/null 2>&1; then
    info "Starting local replica..."
    dfx start --background
    sleep 1
  fi
fi

# 1) Airdrop: Transfer tokens to the user's main account first (as requested)
info "Airdropping funds to user principal..."
AIR_RES=$(dfx canister call "$LEDGER" icrc1_transfer "(record { to = record { owner = principal \"$USER_PRINCIPAL\"; subaccount = null }; amount = $AMOUNT_E8S:nat; fee = null; memo = null; created_at_time = null })" --network "$NETWORK")
info "Airdrop result: $AIR_RES"

# 2) Fetch current episode id and find next stakable episode
EPISODE_RAW=$(dfx canister call "$BACKEND" get_current_episode_id --network "$NETWORK")
CURRENT_EPISODE=$(echo "$EPISODE_RAW" | sed -E 's/[^0-9]*([0-9]+).*/\1/')
# Find next stakable episode (episode_id % 3 == 2)
EPISODE_ID=$((CURRENT_EPISODE + (2 - (CURRENT_EPISODE % 3))))
info "Current episode id: $CURRENT_EPISODE"
info "Using stakable episode id: $EPISODE_ID"

# 3) Get deposit subaccount bytes for the user and episode
SUB_BLOB_RAW=$(dfx canister call "$BACKEND" get_deposit_subaccount "(principal \"$USER_PRINCIPAL\", $EPISODE_ID:nat64)" --network "$NETWORK")
SUB_BLOB_CLEAN=$(echo "$SUB_BLOB_RAW" | sed 's/,$//')
info "Deposit subaccount (clean): $SUB_BLOB_CLEAN"

# 4) Transfer tokens to the deposit subaccount via ICRC-1 ledger (funds from deployer)
info "Transferring amount to deposit subaccount..."
TRANSFER_RES=$(dfx canister call "$LEDGER" icrc1_transfer "(record { to = record { owner = principal \"$BACKEND\"; subaccount = opt $SUB_BLOB_CLEAN }; amount = $AMOUNT_E8S:nat; fee = null; memo = null; created_at_time = null })" --network "$NETWORK")
info "Ledger transfer result: $TRANSFER_RES"

# 5) Call backend deposit to finalize
info "Calling backend deposit(user, episode)..."
DEP_RES=$(dfx canister call "$BACKEND" deposit "(principal \"$USER_PRINCIPAL\", $EPISODE_ID:nat64)" --network "$NETWORK")
info "Backend deposit result: $DEP_RES"

success "Deposit flow completed."
