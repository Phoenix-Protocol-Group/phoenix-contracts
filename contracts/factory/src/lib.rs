#![no_std]
mod contract;
mod storage;
mod utils;

pub mod multihop_contract {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/phoenix_multihop.wasm"
    );
}

#[cfg(test)]
mod tests;
