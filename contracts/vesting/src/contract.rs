use soroban_sdk::{contract, contractimpl, contractmeta, Address, Env, Vec};

use curve::Curve;

use crate::storage::Config;

// Metadata that is added on to the WASM custom section
contractmeta!(key = "Description", val = "Phoenix Protocol Vesting");

#[contract]
pub struct Vesting;

pub trait VestingTrait {
    // Sets the token contract addresses for this pool
    fn initialize(env: Env, admin: Address);

    fn create_vesting_accounts(env: Env, accounts: Vec<Address>);

    fn transfer_token(env: Env, from: Address, to: Address, amount: i128);

    fn transfer_vesting(env: Env, from: Address, to: Address, amount: i128, curve: Curve);

    fn burn(env: Env, amount: i128);

    fn mint(env: Env, sender: Address, to: Address, amount: i128);

    fn update_minter(env: Env, sender: Address, new_minter: Address);

    fn send_tokens_to_contract(env: Env, sender: Address, contract: Address, amount: i128);

    fn add_to_whitelist(env: Env, sender: Address, to_add: Address);

    fn remove_from_whitelist(env: Env, sender: Address, to_remove: Address);

    fn query_config(env: Env) -> Config;
}

#[contractimpl]
impl VestingTrait for Vesting {}
