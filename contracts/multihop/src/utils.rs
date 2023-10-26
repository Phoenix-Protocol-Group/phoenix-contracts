use soroban_sdk::{Env, Symbol, Vec};

use crate::storage::Swap;

pub fn verify_swap( operations: &Vec<Swap>) {
    for (current, next) in operations.iter().zip(operations.iter().skip(1)) {
        if current.ask_asset != next.offer_asset {
            panic!("Multihop: Swap: Provided bad swap order");
        }
    }
}

pub fn verify_reverse_swap(operations: &Vec<Swap>) {
    for (current, next) in operations.iter().zip(operations.iter().skip(1)) {
        if current.offer_asset != next.ask_asset {
            panic!("Multihop: Reverse swap: Provided bad swap order");
        }
    }
}
