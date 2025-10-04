use candid::{encode_args, decode_one, Principal};
use pocket_ic::PocketIc;

pub fn get_stakable_episode(pic: &PocketIc, pool_canister: Principal, caller: Principal) -> u64 {
    let current_episode_bytes = pic
        .query_call(pool_canister, caller, "get_current_episode_id", encode_args(()) .unwrap())
        .unwrap();
    let mut current_episode: u64 = decode_one(&current_episode_bytes).unwrap();

    while current_episode % 3 != 2 {
        current_episode += 1;
    }

    current_episode
}

