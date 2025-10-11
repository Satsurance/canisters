#!/bin/bash

# Deployment script for SATSurance canisters
# This script deploys all canisters one by one and sets their IDs in .env.local

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if dfx is installed
if ! command -v dfx &> /dev/null; then
    print_error "dfx is not installed. Please install dfx first."
    exit 1
fi

# Check if we're in the right directory
if [ ! -f "dfx.json" ]; then
    print_error "dfx.json not found. Please run this script from the project root."
    exit 1
fi

# Get network from command line argument or default to local
NETWORK=${1:-local}
print_status "Deploying to network: $NETWORK"

# Set dfx network
if [ "$NETWORK" != "local" ]; then
    print_status "Setting dfx network to $NETWORK"
    dfx network use $NETWORK
fi

# If local network, verify replica is running
if [ "$NETWORK" = "local" ]; then
    print_status "Verifying local replica is running..."
    if ! curl -s http://127.0.0.1:4943/api/v2/status >/dev/null 2>&1; then
        print_error "Local replica is not running. Please start it with: dfx start --background"
        exit 1
    fi
    print_status "Local replica is running"
fi

# Collect principals for token generation
echo
print_status "=== Token Generation Setup ==="
print_status "Enter principals that should receive initial tokens (one per line, empty line to finish):"

TOKEN_PRINCIPALS=()
TOKEN_AMOUNTS=()
while true; do
    read -p "Principal (or press Enter to finish): " principal
    if [ -z "$principal" ]; then
        break
    fi

    read -p "Amount (default: 100000000000): " amount
    amount=${amount:-100000000000}

    TOKEN_PRINCIPALS+=("$principal")
    TOKEN_AMOUNTS+=("$amount")
    print_success "Added: $principal with amount $amount"
done

if [ ${#TOKEN_PRINCIPALS[@]} -eq 0 ]; then
    print_warning "No principals specified for token generation. Deploying with empty initial balances."
fi


# Ask for executor principal
echo
read -p "Enter executor principal (for slashing/admin operations, or press Enter to use default): " EXECUTOR_PRINCIPAL
if [ -z "$EXECUTOR_PRINCIPAL" ]; then
    EXECUTOR_PRINCIPAL=$(dfx identity get-principal)
    print_status "Using current identity as executor: $EXECUTOR_PRINCIPAL"
else
    print_success "Using executor: $EXECUTOR_PRINCIPAL"
fi

# Ask for pool manager principal
echo
read -p "Enter pool manager principal (or press Enter to use default): " POOL_MANAGER
if [ -z "$POOL_MANAGER" ]; then
    POOL_MANAGER=$(dfx identity get-principal)
    print_status "Using current identity as pool manager: $POOL_MANAGER"
else
    print_success "Using pool manager: $POOL_MANAGER"
fi

# Create .env.local file
ENV_FILE=".env.local"
print_status "Creating $ENV_FILE file"

# Initialize .env.local with network info
cat > $ENV_FILE << EOF
# SATSurance Canister IDs - Generated on $(date)
NETWORK=$NETWORK
EOF

# Function to deploy a canister and get its ID
deploy_canister() {
    local canister_name=$1
    local canister_type=$2
    
    print_status "Deploying $canister_name ($canister_type)..."
    
    # Deploy the canister
    if [ "$canister_type" = "rust" ]; then
        dfx deploy $canister_name --network $NETWORK
    elif [ "$canister_type" = "assets" ]; then
        dfx deploy $canister_name --network $NETWORK
    elif [ "$canister_type" = "custom" ]; then
        dfx deploy $canister_name --network $NETWORK
    else
        print_error "Unknown canister type: $canister_type"
        return 1
    fi
    
    # Get the canister ID
    local canister_id=$(dfx canister id $canister_name --network $NETWORK)
    
    if [ -z "$canister_id" ]; then
        print_error "Failed to get canister ID for $canister_name"
        return 1
    fi
    
    print_success "$canister_name deployed with ID: $canister_id"
    
    # Add to .env.local
    echo "$(echo $canister_name | tr '[:lower:]' '[:upper:]')_ID=$canister_id" >> $ENV_FILE
    
    return 0
}

# Main deployment process
print_status "Starting canister deployment process..."

# 1. Deploy ICRC-1 Ledger (custom canister with init args)
print_status "=== Step 1: Deploying ICRC-1 Ledger ==="

# Build initial_balances argument
INITIAL_BALANCES=""
for i in "${!TOKEN_PRINCIPALS[@]}"; do
    principal="${TOKEN_PRINCIPALS[$i]}"
    amount="${TOKEN_AMOUNTS[$i]}"

    if [ -n "$INITIAL_BALANCES" ]; then
        INITIAL_BALANCES="$INITIAL_BALANCES; "
    fi
    INITIAL_BALANCES="${INITIAL_BALANCES}record { record { owner = principal \"$principal\"; subaccount = null }; $amount }"
done

# Create ICRC-1 ledger with initialization arguments
print_status "Deploying ICRC-1 Ledger with initial balances..."

# Get a temporary principal for minting account (using current identity)
MINTING_PRINCIPAL=$(dfx identity get-principal)

dfx deploy icrc1_ledger --network $NETWORK --argument "(variant {
  Init = record {
    minting_account = record {
      owner = principal \"$MINTING_PRINCIPAL\";
      subaccount = null;
    };
    fee_collector_account = null;
    transfer_fee = 10_000;
    decimals = opt 8;
    max_memo_length = opt 64;
    token_symbol = \"SAT\";
    token_name = \"SATSurance Token\";
    metadata = vec {};
    initial_balances = vec { ${INITIAL_BALANCES} };
    feature_flags = opt record { icrc2 = true };
    maximum_number_of_accounts = null;
    accounts_overflow_trim_quantity = null;
    archive_options = record {
      num_blocks_to_archive = 1000;
      max_transactions_per_response = opt 100;
      trigger_threshold = 2000;
      max_message_size_bytes = opt 1048576;
      cycles_for_archive_creation = opt 1000000000000;
      node_max_memory_size_bytes = opt 33554432;
      controller_id = principal \"$MINTING_PRINCIPAL\";
      more_controller_ids = null;
    };
  }
})"

if [ $? -eq 0 ]; then
    print_success "ICRC-1 Ledger deployment completed"
else
    print_error "ICRC-1 Ledger deployment failed"
    exit 1
fi

# Get ledger canister ID for pool_canister initialization
ICRC1_LEDGER_ID=$(dfx canister id icrc1_ledger --network $NETWORK)
echo "ICRC1_LEDGER_ID=$ICRC1_LEDGER_ID" >> $ENV_FILE

# 2. Deploy Backend Canister (rust) with ledger_id, executor, and pool_manager
print_status "=== Step 2: Deploying Backend Canister ==="
print_status "Deploying pool_canister with ledger: $ICRC1_LEDGER_ID, executor: $EXECUTOR_PRINCIPAL, pool_manager: $POOL_MANAGER"

dfx deploy pool_canister --network $NETWORK --argument "(principal \"$ICRC1_LEDGER_ID\", principal \"$EXECUTOR_PRINCIPAL\", principal \"$POOL_MANAGER\")"

if [ $? -eq 0 ]; then
    print_success "Backend Canister deployment completed"
    BACKEND_ID=$(dfx canister id pool_canister --network $NETWORK)
else
    print_error "Backend Canister deployment failed"
    exit 1
fi

# Create frontend .env file with canister IDs
print_status "Creating frontend environment file with canister IDs..."
cat > src/frontend_canister/.env << EOF
VITE_CANISTER_ID_ICP_CANISTER_BACKEND=$BACKEND_ID
VITE_CANISTER_ID_ICRC1_LEDGER=$ICRC1_LEDGER_ID
VITE_DFX_NETWORK=$NETWORK
EOF
print_success "Frontend .env file created"

# 3. Deploy Frontend (assets)
print_status "=== Step 3: Deploying Frontend ==="

# Check if package.json exists in frontend directory
if [ -f "src/frontend_canister/package.json" ]; then
    cd src/frontend_canister

    # Install dependencies if node_modules doesn't exist
    if [ ! -d "node_modules" ]; then
        print_status "Installing frontend dependencies..."
        npm install
    fi

    # Always rebuild frontend to ensure it has correct canister IDs
    print_status "Building frontend with canister IDs..."
    npm run build

    cd ../..

    if [ ! -d "src/frontend_canister/dist" ]; then
        print_error "Frontend build failed - dist directory not created"
        print_warning "Skipping frontend deployment. You can deploy it later with: dfx deploy frontend_canister --network $NETWORK"
        SKIP_FRONTEND=true
    fi
else
    print_warning "No package.json found in src/frontend_canister"
    print_warning "Skipping frontend deployment. You can deploy it later with: dfx deploy frontend_canister --network $NETWORK"
    SKIP_FRONTEND=true
fi

if [ "$SKIP_FRONTEND" != "true" ]; then
    if deploy_canister "frontend_canister" "assets"; then
        print_success "Frontend deployment completed"
    else
        print_error "Frontend deployment failed"
        exit 1
    fi
else
    print_warning "Frontend canister not deployed"
    FRONTEND_ID="(not deployed)"
fi

# Add additional environment variables
print_status "Adding additional environment variables to $ENV_FILE..."

# Get frontend ID if it was deployed (backend and ledger IDs already set above)
if [ "$SKIP_FRONTEND" != "true" ]; then
    FRONTEND_ID=$(dfx canister id frontend_canister --network $NETWORK)
fi

# Add pool_canister ID and frontend ID
echo "POOL_CANISTER_ID=$BACKEND_ID" >> $ENV_FILE
echo "FRONTEND_CANISTER_ID=$FRONTEND_ID" >> $ENV_FILE

# Add aliases for easier access
cat >> $ENV_FILE << EOF

# Aliases for easier access
LEDGER=$ICRC1_LEDGER_ID
BACKEND=$BACKEND_ID
FRONTEND=$FRONTEND_ID

# Executor principal
EXECUTOR=$EXECUTOR_PRINCIPAL

# Pool Manager principal
POOL_MANAGER=$POOL_MANAGER

# Network configuration
EOF

if [ "$NETWORK" = "local" ]; then
    echo "HOST=http://127.0.0.1:4943" >> $ENV_FILE
else
    echo "HOST=https://icp0.io" >> $ENV_FILE
fi

# Add helpful commands
cat >> $ENV_FILE << EOF

# Token recipients
EOF

# Add token recipients to env file
if [ ${#TOKEN_PRINCIPALS[@]} -gt 0 ]; then
    echo "TOKEN_RECIPIENTS=\"${TOKEN_PRINCIPALS[*]}\"" >> $ENV_FILE
fi

cat >> $ENV_FILE << EOF

# Helpful commands (source this file first: source .env.local)
# Check ledger balance: dfx canister call \$LEDGER icrc1_balance_of "(record { owner = principal \"<principal>\"; subaccount = null })" --network \$NETWORK
# Check pool state: dfx canister call \$BACKEND get_pool_state --network \$NETWORK
# Get current episode: dfx canister call \$BACKEND get_current_episode_id --network \$NETWORK
EOF

print_success "All canisters deployed successfully!"
print_status "Canister IDs saved to $ENV_FILE"

# Display summary
echo
print_status "=== Deployment Summary ==="
echo "Network: $NETWORK"
echo "ICRC-1 Ledger ID: $ICRC1_LEDGER_ID"
echo "Backend Canister ID: $BACKEND_ID"
echo "Frontend Canister ID: $FRONTEND_ID"
echo "Executor Principal: $EXECUTOR_PRINCIPAL"
echo "Pool Manager Principal: $POOL_MANAGER"
echo

# Display token recipients
if [ ${#TOKEN_PRINCIPALS[@]} -gt 0 ]; then
    print_status "=== Initial Token Recipients ==="
    for i in "${!TOKEN_PRINCIPALS[@]}"; do
        echo "  ${TOKEN_PRINCIPALS[$i]}: ${TOKEN_AMOUNTS[$i]} tokens"
    done
    echo
fi

print_status "Environment file created: $ENV_FILE"
print_warning "Remember to source the environment file: source $ENV_FILE"
echo

# Optional: Open frontend URL
if [ "$SKIP_FRONTEND" != "true" ]; then
    if [ "$NETWORK" = "local" ]; then
        FRONTEND_URL="http://$FRONTEND_ID.localhost:4943"
        print_status "Frontend available at: $FRONTEND_URL"
    elif [ "$NETWORK" = "ic" ]; then
        FRONTEND_URL="https://$FRONTEND_ID.ic0.app"
        print_status "Frontend available at: $FRONTEND_URL"
    fi
else
    print_warning "Frontend not deployed. Deploy it later with: dfx deploy frontend_canister --network $NETWORK"
fi

print_success "Deployment completed successfully!"