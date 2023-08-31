#![no_std]
mod contract;
mod error;
mod storage;

mod utils;

#[allow(clippy::too_many_arguments)]
pub mod lp_contract {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/phoenix_pair.wasm"
    );
}

#[cfg(test)]
mod tests;
