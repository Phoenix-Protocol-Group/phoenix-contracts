#![no_std]
mod contract;
mod error;
mod storage;

pub mod stake_contract {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/phoenix_stake.wasm"
    );
}

#[cfg(test)]
mod tests;
