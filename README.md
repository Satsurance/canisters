# `Satsurance Canister`
## Description
Satsurance is an insurance protocol for Bitcoin projects. Liquidity providers (LPs) lock their ckBTC to underwrite risk and earn BTC-denominated yield funded by insurance premiums. Pool managers price Bitcoin protocols based on their risk and balance the pool’s product portfolio.

## Pre-requirements
First, ensure you have the necessary tools installed:
### Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```


### Install DFX (the IC SDK)
```bash
sh -ci "$(curl -fsSL https://internetcomputer.org/install.sh)"
```

### Add wasm32 target for Rust
```bash
rustup target add wasm32-unknown-unknown
```

## Run tests
### Clone the Repository
```bash
git clone https://github.com/Satsurance/canisters.git
cd canisters
```
### Add the pocket-ic

The project requires the pocke_ic binary to run.

#### For macOS / Linux:

1. Download `pocket_ic` from the [official Pocket-IC release page](https://github.com/dfinity/pocketic/releases) .

2. Place the `pocket_ic` binary in the top-level project directory `canisters/` directory (same directory as `dfx.json`).


3. Rename the file to exactly: `pocket_ic`.

4. Make the binary executable:

   ```bash
   chmod +x pocket_ic
   ```
### Build
Compile the project using the following command:
 ```sh
cargo build --target wasm32-unknown-unknown --release
  ```

### Test

To execute the tests, run:

```sh
cargo test
```

## Deployment

The project supports deployment to both local replica and IC mainnet.

### Quick Start

**Deploy to Local Network:**
```bash
./deploy-local.sh
```
This will automatically start a local replica if needed and deploy all canisters.

**Deploy to IC Mainnet:**
```bash
./deploy-mainnet.sh
```
This includes pre-flight checks, interactive identity selection, and safety confirmations before deploying to mainnet.

You can also specify an identity directly:
```bash
./deploy-mainnet.sh --identity production
```

### Advanced Deployment

For more control, use the main deployment script directly:

```bash
# Deploy to local (default)
./deploy.sh

# Deploy to local with explicit network
./deploy.sh local

# Deploy to IC mainnet (with confirmation prompts)
./deploy.sh ic

# Deploy to IC mainnet without confirmation (CI/CD)
./deploy.sh ic --yes

# Deploy to IC using a specific identity
./deploy.sh ic --identity production

# Deploy with specific identity and skip confirmations
./deploy.sh ic --identity production --yes

# Show help
./deploy.sh --help
```

**Identity Management:**

Both deployment scripts support identity selection:
- `deploy-mainnet.sh`: Interactive identity selection by default
- `deploy.sh`: Use `--identity <name>` to specify identity
- Without `--identity` flag, uses current dfx identity

### Network Configuration

The deployment script creates environment files with canister IDs:
- `.env.local` - Root environment file with canister IDs and network info
- `src/frontend_canister/.env` - Frontend environment file with Vite variables

**Local Network:**
- Replica: `http://127.0.0.1:4943`
- Frontend: `http://<canister-id>.localhost:4943`

**IC Mainnet:**
- Frontend: `https://<canister-id>.icp0.io`
- Alternative: `https://<canister-id>.raw.icp0.io`

### Mainnet Prerequisites

Before deploying to mainnet, ensure you have:

1. **DFX Identity configured:**
   ```bash
   dfx identity new my-identity
   dfx identity use my-identity
   ```

2. **Cycles wallet with sufficient cycles:**
   - Get free cycles: https://cyclesfaucet.ic0.app/
   - Or convert ICP to cycles: https://cycles.finance

   Expected costs:
   - Creating 3 canisters: ~300 billion cycles
   - Initial deployment: ~200 billion cycles
   - Total: ~500 billion cycles (0.5T)

3. **Create a cycles wallet (if needed):**
   ```bash
   dfx ledger --network ic create-canister <controller-principal> --amount <icp-amount>
   dfx identity --network ic deploy-wallet <canister-id>
   ```

### Post-Deployment

After successful deployment:

1. **Source the environment file:**
   ```bash
   source .env.local
   ```

2. **Check canister status:**
   ```bash
   dfx canister --network $NETWORK status $BACKEND
   ```

3. **Monitor cycles (mainnet only):**
   ```bash
   dfx canister --network ic status <canister-id>
   ```

4. **Top up canisters as needed:**
   ```bash
   dfx canister --network ic deposit-cycles <amount> <canister-id>
   ```
