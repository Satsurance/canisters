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

# If local network, ensure replica is running
if [ "$NETWORK" = "local" ]; then
    print_status "Ensuring local replica is running..."
    if ! curl -s http://127.0.0.1:4943/api/v2/status >/dev/null 2>&1; then
        print_status "Starting local replica in background"
        dfx start --background
        # wait until status endpoint responds
        for i in {1..30}; do
            if curl -s http://127.0.0.1:4943/api/v2/status >/dev/null 2>&1; then
                break
            fi
            sleep 1
        done
    else
        print_status "Local replica already running"
    fi
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

# 1. Deploy ICRC-1 Ledger (custom canister)
print_status "=== Step 1: Deploying ICRC-1 Ledger ==="
if deploy_canister "icrc1_ledger" "custom"; then
    print_success "ICRC-1 Ledger deployment completed"
else
    print_error "ICRC-1 Ledger deployment failed"
    exit 1
fi

# 2. Deploy Backend Canister (rust)
print_status "=== Step 2: Deploying Backend Canister ==="
if deploy_canister "pool_canister" "rust"; then
    print_success "Backend Canister deployment completed"
else
    print_error "Backend Canister deployment failed"
    exit 1
fi

# 3. Deploy Frontend (assets)
print_status "=== Step 3: Deploying Frontend ==="
if deploy_canister "frontend_canister" "assets"; then
    print_success "Frontend deployment completed"
else
    print_error "Frontend deployment failed"
    exit 1
fi

# Add additional environment variables
print_status "Adding additional environment variables to $ENV_FILE..."

# Get canister IDs for easy access
ICRC1_LEDGER_ID=$(dfx canister id icrc1_ledger --network $NETWORK)
BACKEND_ID=$(dfx canister id pool_canister --network $NETWORK)
FRONTEND_ID=$(dfx canister id frontend_canister --network $NETWORK)

# Add aliases for easier access
cat >> $ENV_FILE << EOF

# Aliases for easier access
LEDGER=$ICRC1_LEDGER_ID
BACKEND=$BACKEND_ID
FRONTEND=$FRONTEND_ID

# Network configuration
EOF

if [ "$NETWORK" = "local" ]; then
    echo "HOST=http://127.0.0.1:4943" >> $ENV_FILE
else
    echo "HOST=https://icp0.io" >> $ENV_FILE
fi

# Add helpful commands
cat >> $ENV_FILE << EOF

# Helpful commands (source this file first: source .env.local)
# Check ledger balance: dfx canister call \$LEDGER icrc1_balance_of "(record { owner = principal \"\$BACKEND\"; subaccount = opt blob \"\$REWARD_SUB\" })" --network \$NETWORK
# Check pool state: dfx canister call \$BACKEND get_pool_state --network \$NETWORK
# Check reward rate: dfx canister call \$BACKEND get_pool_reward_rate --network \$NETWORK
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
echo
print_status "Environment file created: $ENV_FILE"
print_warning "Remember to source the environment file: source $ENV_FILE"
echo

# Optional: Open frontend URL
if [ "$NETWORK" = "local" ]; then
    FRONTEND_URL="http://$FRONTEND_ID.localhost:4943"
    print_status "Frontend available at: $FRONTEND_URL"
elif [ "$NETWORK" = "ic" ]; then
    FRONTEND_URL="https://$FRONTEND_ID.ic0.app"
    print_status "Frontend available at: $FRONTEND_URL"
fi

print_success "Deployment completed successfully!"
