use candid::Principal;
use sha2::{Sha256, Digest};


#[ic_cdk::query]
fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn generate_subaccount(user:Principal,timelock:u64)->[u8;32]{
    let mut hasher = Sha256::new();
    hasher.update(user.as_slice());
    hasher.update(timelock.to_be_bytes());
    hasher.finalize().into()
}
#[ic_cdk::query]
pub fn get_deposit_subaccount(user: Principal, timelock: u64) -> [u8; 32] {
    generate_subaccount(user, timelock)
}

ic_cdk::export_candid!();