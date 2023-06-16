use soroban_sdk::{contractimpl, contractmeta, Address, Bytes, BytesN, Env};

use num_integer::Roots;

use crate::{
    storage::{get_config, save_config, utils, Config},
    token_contract,
};

// Metadata that is added on to the WASM custom section
contractmeta!(
    key = "Description",
    val = "Phoenix Protocol XYK Liquidity Pool"
);

pub struct LiquidityPool;

pub trait LiquidityPoolTrait {
    // Sets the token contract addresses for this pool
    // token_wasm_hash is the WASM hash of the deployed token contract for the pool share token
    fn initialize(
        e: Env,
        token_wasm_hash: BytesN<32>,
        token_a: Address,
        token_b: Address,
        share_token_decimals: u32,
        fee_recipient: Address,
    );

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

    // Returns the address for the pool share token
    fn query_share_token_address(e: Env) -> Address;
}

#[contractimpl]
impl LiquidityPoolTrait for LiquidityPool {
    fn initialize(
        env: Env,
        token_wasm_hash: BytesN<32>,
        token_a: Address,
        token_b: Address,
        share_token_decimals: u32,
        _fee_recipient: Address,
    ) {
        // Token order validation to make sure only one instance of a pool can exist
        if token_a >= token_b {
            panic!("token_a must be less than token_b");
        }

        // deploy token contract
        let share_token_address =
            utils::deploy_token_contract(&env, &token_wasm_hash, &token_a, &token_b);
        token_contract::Client::new(&env, &share_token_address).initialize(
            // admin
            &env.current_contract_address(),
            // number of decimals on the share token
            &share_token_decimals,
            // name
            &Bytes::from_slice(&env, b"Pool Share Token"),
            // symbol
            &Bytes::from_slice(&env, b"POOL"),
        );

        let config = Config {
            token_a,
            token_b,
            share_token: share_token_address,
        };
        save_config(&env, config);
        utils::save_total_shares(&env, 0);
        utils::save_pool_balance_a(&env, 0);
        utils::save_pool_balance_b(&env, 0);
    }

    fn deposit(env: Env, to: Address, desired_a: i128, min_a: i128, desired_b: i128, min_b: i128) {
        // Depositor needs to authorize the deposit
        to.require_auth();

        let pool_balance_a = utils::get_pool_balance_a(&env);
        let pool_balance_b = utils::get_pool_balance_b(&env);

        // Calculate deposit amounts
        let amounts = utils::get_deposit_amounts(
            desired_a,
            min_a,
            desired_b,
            min_b,
            pool_balance_a,
            pool_balance_b,
        );

        let config = get_config(&env);

        let token_a_client = token_contract::Client::new(&env, &config.token_a);
        let token_b_client = token_contract::Client::new(&env, &config.token_b);

        // Move tokens from client's wallet to the contract
        token_a_client.transfer(&to, &env.current_contract_address(), &amounts.0);
        token_b_client.transfer(&to, &env.current_contract_address(), &amounts.1);

        // Now calculate how many new pool shares to mint
        let balance_a = utils::get_balance(&env, config.token_a);
        let balance_b = utils::get_balance(&env, config.token_b);
        let total_shares = utils::get_total_shares(&env);

        let new_total_shares = if pool_balance_a > 0 && pool_balance_b > 0 {
            let shares_a = (balance_a * total_shares) / pool_balance_a;
            let shares_b = (balance_b * total_shares) / pool_balance_b;
            shares_a.min(shares_b)
        } else {
            (balance_a * balance_b).sqrt()
        };

        utils::mint_shares(
            &env,
            config.share_token,
            to,
            new_total_shares - total_shares,
        );
        utils::save_pool_balance_a(&env, balance_a);
        utils::save_pool_balance_b(&env, balance_b);
    }

    fn swap(_e: Env, _to: Address, _buy_a: bool, _out: i128, _in_max: i128) {
        unimplemented!()
    }

    fn withdraw(
        _e: Env,
        _to: Address,
        _share_amount: i128,
        _min_a: i128,
        _min_b: i128,
    ) -> (i128, i128) {
        unimplemented!()
    }

    // Queries

    fn query_share_token_address(env: Env) -> Address {
        get_config(&env).share_token
    }
}
