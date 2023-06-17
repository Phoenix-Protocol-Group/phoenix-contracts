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
    );

    // Deposits token_a and token_b. Also mints pool shares for the "to" Identifier. The amount minted
    // is determined based on the difference between the reserves stored by this contract, and
    // the actual balance of token_a and token_b for this contract.
    fn provide_liquidity(
        e: Env,
        to: Address,
        desired_a: u128,
        min_a: u128,
        desired_b: u128,
        min_b: u128,
    );

    // If "buy_a" is true, the swap will buy token_a and sell token_b. This is flipped if "buy_a" is false.
    // "out" is the amount being bought, with in_max being a safety to make sure you receive at least that amount.
    // swap will transfer the selling token "to" to this contract, and then the contract will transfer the buying token to "to".
    fn swap(e: Env, to: Address, buy_a: bool, out: u128, in_max: u128);

    // transfers share_amount of pool share tokens to this contract, burns all pools share tokens in this contracts, and sends the
    // corresponding amount of token_a and token_b to "to".
    // Returns amount of both tokens withdrawn
    fn withdraw_liquidity(
        e: Env,
        to: Address,
        share_amount: u128,
        min_a: u128,
        min_b: u128,
    ) -> (u128, u128);

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

    fn provide_liquidity(
        env: Env,
        to: Address,
        desired_a: u128,
        min_a: u128,
        desired_b: u128,
        min_b: u128,
    ) {
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

        // TODO: Add slippage_tolerance to configuration
        assert_slippage_tolerance(
            None,
            &[amounts.0, amounts.1],
            &[pool_balance_a, pool_balance_b],
        );

        let config = get_config(&env);

        let token_a_client = token_contract::Client::new(&env, &config.token_a);
        let token_b_client = token_contract::Client::new(&env, &config.token_b);

        // Move tokens from client's wallet to the contract
        token_a_client.transfer(&to, &env.current_contract_address(), &(amounts.0 as i128));
        token_b_client.transfer(&to, &env.current_contract_address(), &(amounts.1 as i128));

        // Now calculate how many new pool shares to mint
        let balance_a = utils::get_balance(&env, config.token_a) as u128;
        let balance_b = utils::get_balance(&env, config.token_b) as u128;
        let total_shares = utils::get_total_shares(&env);

        let new_total_shares = if pool_balance_a > 0 && pool_balance_b > 0 {
            let shares_a = (balance_a * total_shares) / pool_balance_a;
            let shares_b = (balance_b * total_shares) / pool_balance_b;
            shares_a.min(shares_b)
        } else {
            // In case of empty pool, just produce X*Y shares
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

    fn swap(_e: Env, _to: Address, _buy_a: bool, _out: u128, _in_max: u128) {
        unimplemented!()
    }

    fn withdraw_liquidity(
        _e: Env,
        _to: Address,
        _share_amount: u128,
        _min_a: u128,
        _min_b: u128,
    ) -> (u128, u128) {
        unimplemented!()
    }

    // Queries

    fn query_share_token_address(env: Env) -> Address {
        get_config(&env).share_token
    }
}

pub fn assert_slippage_tolerance(
    slippage_tolerance: Option<u128>,
    deposits: &[u128; 2],
    pools: &[u128; 2],
) {
    let default_slippage = 100; // Representing 1.00 (100%) as the default slippage tolerance
    let max_allowed_slippage = 500; // Representing 5.00 (500%) as the maximum allowed slippage tolerance

    let slippage_tolerance = slippage_tolerance.unwrap_or(default_slippage);
    if slippage_tolerance > max_allowed_slippage {
        panic!(
            "Slippage tolerance {} exceeds the maximum allowed value",
            slippage_tolerance
        );
    }

    let slippage_tolerance = slippage_tolerance * 100; // Converting to a percentage value
    let one_minus_slippage_tolerance = 10000 - slippage_tolerance;
    let deposits: [u128; 2] = [deposits[0], deposits[1]];
    let pools: [u128; 2] = [pools[0], pools[1]];

    // Ensure each price does not change more than what the slippage tolerance allows
    if deposits[0] * pools[1] * one_minus_slippage_tolerance > deposits[1] * pools[0] * 10000
        || deposits[1] * pools[0] * one_minus_slippage_tolerance > deposits[0] * pools[1] * 10000
    {
        panic!(
            "Slippage tolerance violated. Deposits: {:?}, Pools: {:?}",
            deposits, pools
        );
    }
}
