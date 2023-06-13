#![no_std]
use soroban_sdk::{
    contractimpl, contractmeta, Address, Bytes, BytesN, ConversionError, Env, RawVal, TryFromVal,
};

pub struct LiquidityPool;

// Metadata that is added on to the WASM custom section
contractmeta!(
    key = "Description",
    val = "Phoenix Protocol XYK Liquidity Pool"
);

pub trait LiquidityPoolTrait {
    // Sets the token contract addresses for this pool
    // token_wasm_hash is the WASM hash of the deployed token contract for the pool share token
    fn initialize(e: Env, token_wasm_hash: BytesN<32>, token_a: Address, token_b: Address, fee_recipient: Address);

    // Returns the token contract address for the pool share token
    fn share_id(e: Env) -> Address;

    // Deposits token_a and token_b. Also mints pool shares for the "to" Identifier. The amount minted
    // is determined based on the difference between the reserves stored by this contract, and
    // the actual balance of token_a and token_b for this contract.
    fn deposit(e: Env, to: Address, desired_a: i128, min_a: i128, desired_b: i128, min_b: i128);

    // If "buy_a" is true, the swap will buy token_a and sell token_b. This is flipped if "buy_a" is false.
    // "out" is the amount being bought, with in_max being a safety to make sure you receive at least that amount.
    // swap will transfer the selling token "to" to this contract, and then the contract will transfer the buying token to "to".
    fn swap(e: Env, to: Address, buy_a: bool, out: i128, in_max: i128);

    // transfers share_amount of pool share tokens to this contract, burns all pools share tokens in this contracts, and sends the
    // corresponding amount of token_a and token_b to "to".
    // Returns amount of both tokens withdrawn
    fn withdraw(e: Env, to: Address, share_amount: i128, min_a: i128, min_b: i128) -> (i128, i128);
}

#[contractimpl]
impl LiquidityPool {
    fn initialize(e: Env, token_wasm_hash: BytesN<32>, token_a: Address, token_b: Address) {
        unimplemented!();
    }

    fn share_id(e: Env) -> Address {
        unimplemented!()
    }

    fn deposit(e: Env, to: Address, desired_a: i128, min_a: i128, desired_b: i128, min_b: i128) {
        unimplemented!()
    }

    fn swap(e: Env, to: Address, buy_a: bool, out: i128, in_max: i128) {
        unimplemented!()
    }

    fn withdraw(e: Env, to: Address, share_amount: i128, min_a: i128, min_b: i128) -> (i128, i128) {
        unimplemented!()
    }
}
