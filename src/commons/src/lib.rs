// Always available - no external dependencies beyond core crates
pub mod types;
pub use types::*;

// Test utilities - only available when NOT targeting wasm32
// These require dev-dependencies (pocket-ic, pool_canister, claim_canister)
#[cfg(not(target_arch = "wasm32"))]
pub mod canister_client;
#[cfg(not(target_arch = "wasm32"))]
pub mod clients;
#[cfg(not(target_arch = "wasm32"))]
pub mod macros;
#[cfg(not(target_arch = "wasm32"))]
pub mod utils;

#[cfg(not(target_arch = "wasm32"))]
pub use canister_client::CanisterClient;
#[cfg(not(target_arch = "wasm32"))]
pub use clients::*;
#[cfg(not(target_arch = "wasm32"))]
pub use utils::*;
