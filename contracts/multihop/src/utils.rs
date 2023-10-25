use soroban_sdk::{Env, Symbol, Vec};

use crate::storage::Swap;

pub fn verify_operations(env: &Env, operations: &Vec<Swap>) -> Option<Symbol> {
    if operations.is_empty() {
        return Some(Symbol::new(env, "operations_empty"));
    }

    for i in 0..operations.len() - 1 {
        if operations.len() > 1
            && operations.get(i).unwrap().ask_asset != operations.get(i + 1).unwrap().offer_asset
        {
            return Some(Symbol::new(env, "bad_swaps"));
        }
    }
    None
}
