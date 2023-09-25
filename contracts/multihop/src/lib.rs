#![no_std]
mod contract;

mod error;

mod storage;

pub mod factory_contract {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/phoenix_factory.wasm"
    );
}

#[cfg(test)]
mod tests;
