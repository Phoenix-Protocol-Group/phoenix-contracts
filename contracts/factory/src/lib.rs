#![no_std]
mod contract;
mod error;
mod storage;

mod utils;

pub mod lp_contract {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/phoenix_pair.wasm"
    );
}
