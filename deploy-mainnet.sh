#!/bin/bash

# Convenience script for deploying to IC mainnet
# This is a wrapper around deploy.sh with safety checks for mainnet deployment
#
# Usage:
#   ./deploy-mainnet.sh [options]
#
# Options:
#   --identity <name>   Use specific dfx identity (skip interactive selection)
#   --yes               Skip all confirmation prompts
#   --help              Show this help message
#
# Examples:
#   ./deploy-mainnet.sh                      # Interactive: choose identity and confirm
#   ./deploy-mainnet.sh --identity production # Use 'production' identity
#   ./deploy-mainnet.sh --yes                # Skip confirmations (use current identity)

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Parse command-line arguments
SPECIFIED_IDENTITY=""
EXTRA_ARGS=()

while [[ $# -gt 0 ]]; do
    case $1 in
        --identity)
            SPECIFIED_IDENTITY="$2"
            shift 2
            ;;
        --help|-h)
            echo "Convenience script for deploying to IC mainnet"
            echo
            echo "Usage:"
            echo "  ./deploy-mainnet.sh [options]"
            echo
            echo "Options:"
            echo "  --identity <name>   Use specific dfx identity (skip interactive selection)"
            echo "  --yes               Skip all confirmation prompts"
            echo "  --help              Show this help message"
            echo
            echo "Examples:"
            echo "  ./deploy-mainnet.sh                      # Interactive: choose identity and confirm"
            echo "  ./deploy-mainnet.sh --identity production # Use 'production' identity"
            echo "  ./deploy-mainnet.sh --yes                # Skip confirmations (use current identity)"
            exit 0
            ;;
        *)
            EXTRA_ARGS+=("$1")
            shift
            ;;
    esac
done

echo -e "${YELLOW}[WARNING]${NC} Preparing to deploy to Internet Computer MAINNET"
echo

# Pre-flight checks
echo -e "${BLUE}[INFO]${NC} Running pre-flight checks..."

# Check if dfx is installed
if ! command -v dfx &> /dev/null; then
    echo -e "${RED}[ERROR]${NC} dfx is not installed. Please install dfx first."
    exit 1
fi

# Handle identity selection
if [ -n "$SPECIFIED_IDENTITY" ]; then
    # Identity specified via command line
    echo -e "${BLUE}[INFO]${NC} Using specified identity: $SPECIFIED_IDENTITY"
    if dfx identity use "$SPECIFIED_IDENTITY" 2>/dev/null; then
        IDENTITY=$SPECIFIED_IDENTITY
        echo -e "${GREEN}[SUCCESS]${NC} Switched to identity: $IDENTITY"
    else
        echo -e "${RED}[ERROR]${NC} Failed to switch to identity '$SPECIFIED_IDENTITY'"
        echo "Available identities:"
        dfx identity list
        exit 1
    fi
else
    # Interactive identity selection
    echo -e "${BLUE}[INFO]${NC} Available dfx identities:"
    echo
    dfx identity list
    echo

    CURRENT_IDENTITY=$(dfx identity whoami 2>/dev/null || echo "")
    if [ -z "$CURRENT_IDENTITY" ]; then
        echo -e "${RED}[ERROR]${NC} No dfx identity configured."
        echo "Please create one with: dfx identity new <name>"
        exit 1
    fi

    echo -e "${BLUE}[INFO]${NC} Current identity: ${YELLOW}$CURRENT_IDENTITY${NC}"
    echo

    read -p "Enter identity to use (or press Enter to use '$CURRENT_IDENTITY'): " SELECTED_IDENTITY

    if [ -z "$SELECTED_IDENTITY" ]; then
        IDENTITY=$CURRENT_IDENTITY
        echo -e "${GREEN}[SUCCESS]${NC} Using current identity: $IDENTITY"
    else
        # Verify the selected identity exists
        # Use grep -E for extended regex, matching either "name" or "name *" (for current identity)
        if dfx identity list | grep -Eq "^${SELECTED_IDENTITY}$|^${SELECTED_IDENTITY} \*$"; then
            dfx identity use "$SELECTED_IDENTITY"
            IDENTITY=$SELECTED_IDENTITY
            echo -e "${GREEN}[SUCCESS]${NC} Switched to identity: $IDENTITY"
        else
            echo -e "${RED}[ERROR]${NC} Identity '$SELECTED_IDENTITY' not found."
            echo "Available identities:"
            dfx identity list
            exit 1
        fi
    fi
fi

# Check for cycles wallet
WALLET_CHECK=$(dfx wallet --network ic balance 2>&1 || echo "no-wallet")
if echo "$WALLET_CHECK" | grep -q "no-wallet\|not found\|error"; then
    echo
    echo -e "${YELLOW}[WARNING]${NC} No cycles wallet detected for mainnet!"
    echo
    echo "To deploy to mainnet, you need a cycles wallet with sufficient cycles."
    echo
    echo "Options:"
    echo "1. If you have ICP in your account, convert it to cycles:"
    echo "   dfx ledger --network ic create-canister <controller-principal> --amount <icp-amount>"
    echo "   dfx identity --network ic deploy-wallet <canister-id>"
    echo
    echo "2. Get cycles from a cycles provider like:"
    echo "   - https://cyclesfaucet.ic0.app/ (free cycles for development)"
    echo "   - https://cycles.finance (convert ICP to cycles)"
    echo
    read -p "Do you want to continue anyway? (yes/no): " CONTINUE
    if [ "$CONTINUE" != "yes" ]; then
        echo "Deployment cancelled"
        exit 0
    fi
else
    echo -e "${GREEN}[SUCCESS]${NC} Cycles wallet found: $WALLET_CHECK"
fi

echo
echo -e "${BLUE}[INFO]${NC} All pre-flight checks passed"
echo

# Forward all arguments to the main deploy script
# Note: The main script will still ask for confirmation unless --yes is passed
./deploy.sh ic "${EXTRA_ARGS[@]}"
