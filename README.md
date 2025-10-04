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
