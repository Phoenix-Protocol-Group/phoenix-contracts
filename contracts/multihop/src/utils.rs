use soroban_sdk::Vec;

use crate::storage::Swap;

pub fn verify_operations(operations: &Vec<Swap>) {
    if operations.is_empty() {
        panic!("Multihop: Operations empty");
    }

    for i in 0..operations.len() - 1 {
        if operations.get(i).unwrap().ask_asset != operations.get(i + 1).unwrap().offer_asset {
            panic!("Multihop: Provided bad swap order")
        }
    }
}
