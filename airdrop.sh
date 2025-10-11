#!/bin/bash

# Airdrop tokens to any user principal
# Usage: ./airdrop.sh <principal> [amount_in_tokens]
# Example: ./airdrop.sh n4iqw-rhmtl-rtvgq-cb67w-nxpf6-menji-dutmr-rwrop-2tmkt-pl3lp-6qe 10

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

info() { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; }
warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }

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
: "${NETWORK:=local}"

# Get principal from argument or prompt
USER_PRINCIPAL=${1:-}
if [ -z "$USER_PRINCIPAL" ]; then
  read -rp "Enter user principal to airdrop tokens to: " USER_PRINCIPAL
fi

# Get amount in tokens (default 10 tokens)
# 1 token = 10^8 smallest units (8 decimals)
AMOUNT_TOKENS=${2:-10}
AMOUNT_E8S=$((AMOUNT_TOKENS * 100000000))

info "Airdropping to principal: $USER_PRINCIPAL"
info "Amount: $AMOUNT_TOKENS tokens ($AMOUNT_E8S base units)"

# Ensure replica for local
if [ "${NETWORK}" = "local" ]; then
  if ! curl -s http://127.0.0.1:4943/api/v2/status >/dev/null 2>&1; then
    info "Starting local replica..."
    dfx start --background
    sleep 2
  fi
fi

# Check minter (deployer) balance
MINTER=$(dfx identity get-principal)
MINTER_BALANCE=$(dfx canister call "$LEDGER" icrc1_balance_of "(record { owner = principal \"$MINTER\"; subaccount = null })" --network "$NETWORK" | grep -o '[0-9_]*' | tr -d '_')

info "Your (minter) balance: $MINTER_BALANCE base units"

if [ "$MINTER_BALANCE" -lt "$AMOUNT_E8S" ]; then
  error "Insufficient balance! You have $MINTER_BALANCE but need $AMOUNT_E8S"
  exit 1
fi

# Transfer tokens to user
info "Transferring $AMOUNT_TOKENS tokens to $USER_PRINCIPAL..."
TRANSFER_RES=$(dfx canister call "$LEDGER" icrc1_transfer "(record {
  to = record {
    owner = principal \"$USER_PRINCIPAL\";
    subaccount = null
  };
  amount = $AMOUNT_E8S:nat;
  fee = null;
  memo = null;
  created_at_time = null
})" --network "$NETWORK")

info "Transfer result: $TRANSFER_RES"

# Check if transfer was successful
if echo "$TRANSFER_RES" | grep -q "Ok"; then
  # Get the user's new balance
  USER_BALANCE=$(dfx canister call "$LEDGER" icrc1_balance_of "(record { owner = principal \"$USER_PRINCIPAL\"; subaccount = null })" --network "$NETWORK")
  success "Airdrop successful!"
  info "User balance: $USER_BALANCE"
else
  error "Airdrop failed!"
  exit 1
fi
