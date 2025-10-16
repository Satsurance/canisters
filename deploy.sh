#!/bin/bash

# Deployment script for SATSurance canisters
# This script deploys all canisters one by one and sets their IDs in .env.local
#
# Usage:
#   ./deploy.sh [network] [options]
#
# Arguments:
#   network     Network to deploy to: 'local' or 'ic' (default: local)
#
# Options:
#   --yes               Skip confirmation prompts (useful for CI/CD)
#   --identity <name>   Use specific dfx identity for deployment
#   --help              Show this help message
#
# Examples:
#   ./deploy.sh                          # Deploy to local network
#   ./deploy.sh local                    # Deploy to local network
#   ./deploy.sh ic                       # Deploy to IC mainnet (with confirmations)
#   ./deploy.sh ic --yes                 # Deploy to IC mainnet (skip confirmations)
#   ./deploy.sh ic --identity production # Deploy to IC using 'production' identity

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

# Function to show help
show_help() {
    echo "Deployment script for SATSurance canisters"
    echo
    echo "Usage:"
    echo "  ./deploy.sh [network] [options]"
    echo
    echo "Arguments:"
    echo "  network     Network to deploy to: 'local' or 'ic' (default: local)"
    echo
    echo "Options:"
    echo "  --yes               Skip confirmation prompts (useful for CI/CD)"
    echo "  --identity <name>   Use specific dfx identity for deployment"
    echo "  --help              Show this help message"
    echo
    echo "Examples:"
    echo "  ./deploy.sh                          # Deploy to local network"
    echo "  ./deploy.sh local                    # Deploy to local network"
    echo "  ./deploy.sh ic                       # Deploy to IC mainnet (with confirmations)"
    echo "  ./deploy.sh ic --yes                 # Deploy to IC mainnet (skip confirmations)"
    echo "  ./deploy.sh ic --identity production # Deploy to IC using 'production' identity"
    exit 0
}

# Parse command line arguments
NETWORK="local"
SKIP_CONFIRMATION=false
SPECIFIED_IDENTITY=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --help|-h)
            show_help
            ;;
        --yes|-y)
            SKIP_CONFIRMATION=true
            shift
            ;;
        --identity)
            SPECIFIED_IDENTITY="$2"
            shift 2
            ;;
        local|ic)
            NETWORK=$1
            shift
            ;;
        *)
            print_error "Unknown argument: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

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

print_status "Deploying to network: $NETWORK"

# Validate network
if [ "$NETWORK" != "local" ] && [ "$NETWORK" != "ic" ]; then
    print_error "Invalid network: $NETWORK"
    print_error "Valid networks are: 'local' or 'ic'"
    exit 1
fi

# Handle identity selection if specified
if [ -n "$SPECIFIED_IDENTITY" ]; then
    print_status "Switching to identity: $SPECIFIED_IDENTITY"
    if dfx identity use "$SPECIFIED_IDENTITY" 2>/dev/null; then
        print_success "Using identity: $SPECIFIED_IDENTITY"
    else
        print_error "Failed to switch to identity '$SPECIFIED_IDENTITY'"
        print_error "Available identities:"
        dfx identity list
        exit 1
    fi
fi

# Mainnet-specific checks and confirmations
if [ "$NETWORK" = "ic" ]; then
    print_warning "=== MAINNET DEPLOYMENT ==="
    print_warning "You are about to deploy to the Internet Computer mainnet!"
    echo

    # Check if identity is configured
    IDENTITY=$(dfx identity whoami 2>/dev/null || echo "")
    if [ -z "$IDENTITY" ]; then
        print_error "No dfx identity configured. Please create one with: dfx identity new <name>"
        exit 1
    fi
    print_status "Using identity: $IDENTITY"

    # Get identity principal
    IDENTITY_PRINCIPAL=$(dfx identity get-principal 2>/dev/null || echo "")
    if [ -z "$IDENTITY_PRINCIPAL" ]; then
        print_error "Failed to get identity principal"
        exit 1
    fi
    print_status "Identity principal: $IDENTITY_PRINCIPAL"

    # Check cycles balance (if wallet exists)
    print_status "Checking cycles balance..."
    WALLET_BALANCE=$(dfx wallet --network ic balance 2>/dev/null || echo "")
    if [ -n "$WALLET_BALANCE" ]; then
        print_status "Current wallet balance: $WALLET_BALANCE"

        # Extract cycles amount (remove "TC" or "T" suffix)
        CYCLES_AMOUNT=$(echo "$WALLET_BALANCE" | grep -oE '[0-9.]+' | head -1)

        # Warn if balance seems low (less than 1T cycles)
        if [ -n "$CYCLES_AMOUNT" ]; then
            if (( $(echo "$CYCLES_AMOUNT < 1" | bc -l 2>/dev/null || echo 0) )); then
                print_warning "Wallet balance is low! You may not have enough cycles for deployment."
                print_warning "Each canister creation costs ~100B cycles, plus deployment costs."
            fi
        fi
    else
        print_warning "Could not check wallet balance. Make sure you have sufficient cycles."
        print_warning "You can create a cycles wallet with: dfx identity deploy-wallet"
    fi

    # Show estimated costs
    echo
    print_status "=== Estimated Deployment Costs ==="
    echo "  - Creating 3 canisters: ~300 billion cycles"
    echo "  - Initial canister storage: ~100 billion cycles"
    echo "  - Total estimated: ~500 billion cycles (0.5T)"
    echo

    # Confirmation prompt (unless --yes flag is used)
    if [ "$SKIP_CONFIRMATION" = false ]; then
        echo
        print_warning "Are you sure you want to deploy to mainnet?"
        read -p "Type 'yes' to continue: " CONFIRM
        if [ "$CONFIRM" != "yes" ]; then
            print_status "Deployment cancelled by user"
            exit 0
        fi
    else
        print_status "Skipping confirmation (--yes flag provided)"
    fi
    echo
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
# print_status "=== Step 1: Deploying ICRC-1 Ledger ==="

# # Build initial_balances argument
# INITIAL_BALANCES=""
# for i in "${!TOKEN_PRINCIPALS[@]}"; do
#     principal="${TOKEN_PRINCIPALS[$i]}"
#     amount="${TOKEN_AMOUNTS[$i]}"

#     if [ -n "$INITIAL_BALANCES" ]; then
#         INITIAL_BALANCES="$INITIAL_BALANCES; "
#     fi
#     INITIAL_BALANCES="${INITIAL_BALANCES}record { record { owner = principal \"$principal\"; subaccount = null }; $amount }"
# done

# # Create ICRC-1 ledger with initialization arguments
# print_status "Deploying ICRC-1 Ledger with initial balances..."

# # Get a temporary principal for minting account (using current identity)
# MINTING_PRINCIPAL=$(dfx identity get-principal)

# dfx deploy icrc1_ledger --network $NETWORK --argument "(variant {
#   Init = record {
#     minting_account = record {
#       owner = principal \"$MINTING_PRINCIPAL\";
#       subaccount = null;
#     };
#     fee_collector_account = null;
#     transfer_fee = 10;
#     decimals = opt 8;
#     max_memo_length = opt 64;
#     token_symbol = \"FckBTC\";
#     token_name = \"Fake ckBTC\";
#     metadata = vec {};
#     initial_balances = vec { ${INITIAL_BALANCES} };
#     feature_flags = opt record { icrc2 = true };
#     maximum_number_of_accounts = null;
#     accounts_overflow_trim_quantity = null;
#     archive_options = record {
#       num_blocks_to_archive = 1000;
#       max_transactions_per_response = opt 100;
#       trigger_threshold = 2000;
#       max_message_size_bytes = opt 1048576;
#       cycles_for_archive_creation = opt 1000000000000;
#       node_max_memory_size_bytes = opt 33554432;
#       controller_id = principal \"$MINTING_PRINCIPAL\";
#       more_controller_ids = null;
#     };
#   }
# })"

# if [ $? -eq 0 ]; then
#     print_success "ICRC-1 Ledger deployment completed"
# else
#     print_error "ICRC-1 Ledger deployment failed"
#     exit 1
# fi

# Get ledger canister ID for pool_canister initialization
ICRC1_LEDGER_ID=$(dfx canister id icrc1_ledger --network $NETWORK)
echo "ICRC1_LEDGER_ID=$ICRC1_LEDGER_ID" >> $ENV_FILE

# 2. Deploy Backend Canister (rust) with ledger_id, executor, and pool_manager
print_status "=== Step 2: Deploying Backend Canister ==="
print_status "Deploying pool_canister with ledger: $ICRC1_LEDGER_ID, executor: $EXECUTOR_PRINCIPAL, pool_manager: $POOL_MANAGER"

# dfx deploy pool_canister --network $NETWORK --argument "(principal \"$ICRC1_LEDGER_ID\", principal \"$EXECUTOR_PRINCIPAL\", principal \"$POOL_MANAGER\")"

if [ $? -eq 0 ]; then
    print_success "Backend Canister deployment completed"
    BACKEND_ID=$(dfx canister id pool_canister --network $NETWORK)
else
    print_error "Backend Canister deployment failed"
    exit 1
fi

# Create frontend .env file with canister IDs
print_status "Creating frontend environment file with canister IDs..."

# Determine the suffix based on network
if [ "$NETWORK" = "local" ]; then
    ENV_SUFFIX="LOCAL"
elif [ "$NETWORK" = "ic" ]; then
    ENV_SUFFIX="MAINNET"
else
    ENV_SUFFIX="LOCAL"
fi

# Read existing .env file if it exists to preserve other network's variables
FRONTEND_ENV_FILE="src/frontend_canister/.env"
TEMP_ENV_FILE="src/frontend_canister/.env.tmp"

if [ -f "$FRONTEND_ENV_FILE" ]; then
    # Copy existing file, excluding variables for the current network
    grep -v "VITE_CANISTER_ID_ICP_CANISTER_BACKEND_${ENV_SUFFIX}" "$FRONTEND_ENV_FILE" | \
    grep -v "VITE_CANISTER_ID_ICRC1_LEDGER_${ENV_SUFFIX}" > "$TEMP_ENV_FILE" || true
else
    touch "$TEMP_ENV_FILE"
fi

# Append new variables for the current network
cat >> "$TEMP_ENV_FILE" << EOF
VITE_CANISTER_ID_ICP_CANISTER_BACKEND_${ENV_SUFFIX}=$BACKEND_ID
VITE_CANISTER_ID_ICRC1_LEDGER_${ENV_SUFFIX}=$ICRC1_LEDGER_ID
EOF

# Move temp file to actual .env file
mv "$TEMP_ENV_FILE" "$FRONTEND_ENV_FILE"

print_success "Frontend .env file updated with $ENV_SUFFIX canister IDs"

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
elif [ "$NETWORK" = "ic" ]; then
    echo "HOST=https://icp0.io" >> $ENV_FILE
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
        FRONTEND_URL="https://$FRONTEND_ID.icp0.io"
        print_status "Frontend available at: $FRONTEND_URL"
        print_status "Alternative URL: https://$FRONTEND_ID.raw.icp0.io"
    fi
else
    print_warning "Frontend not deployed. Deploy it later with: dfx deploy frontend_canister --network $NETWORK"
fi

# Add post-deployment notes for mainnet
if [ "$NETWORK" = "ic" ]; then
    echo
    print_warning "=== IMPORTANT: Mainnet Deployment Notes ==="
    echo "1. Save your canister IDs (stored in .env.local)"
    echo "2. Consider setting up monitoring for your canisters"
    echo "3. Top up your canisters with cycles regularly"
    echo "4. Check canister status: dfx canister --network ic status <canister-id>"
    echo "5. Monitor cycles: dfx canister --network ic status <canister-id>"
    echo
fi

print_success "Deployment completed successfully!"