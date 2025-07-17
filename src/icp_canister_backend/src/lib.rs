

#[ic_cdk::query]
fn add(a: i32, b: i32) -> i32 {
    a + b
}

ic_cdk::export_candid!();