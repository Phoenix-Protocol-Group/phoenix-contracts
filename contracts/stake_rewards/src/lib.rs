#![no_std]
mod contract;
mod distribution;
mod error;
mod msg;
mod storage;

pub const TOKEN_PER_POWER: i32 = 1_000;

#[allow(clippy::too_many_arguments)]
pub mod stake_contract {
    // The import will code generate:
    // - A ContractClient type that can be used to invoke functions on the contract.
    // - Any types in the contract that were annotated with #[contracttype].
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/phoenix_stake.wasm"
    );
}

#[allow(clippy::too_many_arguments)]
pub mod token_contract {
    // The import will code generate:
    // - A ContractClient type that can be used to invoke functions on the contract.
    // - Any types in the contract that were annotated with #[contracttype].
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/soroban_token_contract.wasm"
    );
}

#[cfg(test)]
mod tests;
