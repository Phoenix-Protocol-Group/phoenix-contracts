use soroban_sdk::{contractimpl, contractmeta, Address, Bytes, BytesN, Env};

use num_integer::Roots;

use crate::{
    storage::{get_config, save_config, utils, Config},
    token_contract,
};
use decimal::Decimal;

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
        env: Env,
        token_wasm_hash: BytesN<32>,
        token_a: Address,
        token_b: Address,
        share_token_decimals: u32,
    );

    // Deposits token_a and token_b. Also mints pool shares for the "to" Identifier. The amount minted
    // is determined based on the difference between the reserves stored by this contract, and
    // the actual balance of token_a and token_b for this contract.
    fn provide_liquidity(
        env: Env,
        depositor: Address,
        desired_a: u128,
        min_a: u128,
        desired_b: u128,
        min_b: u128,
    );

    // If "buy_a" is true, the swap will buy token_a and sell token_b. This is flipped if "buy_a" is false.
    // "out" is the amount being bought, with in_max being a safety to make sure you receive at least that amount.
    // swap will transfer the selling token "to" to this contract, and then the contract will transfer the buying token to "to".
    fn swap(
        env: Env,
        sender: Address,
        sell_a: bool,
        sell_amount: u128,
        belief_price: Option<u64>,
        max_spread: u64,
    );

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
        depositor: Address,
        desired_a: u128,
        min_a: u128,
        desired_b: u128,
        min_b: u128,
    ) {
        // Depositor needs to authorize the deposit
        depositor.require_auth();

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
        token_a_client.transfer(
            &depositor,
            &env.current_contract_address(),
            &(amounts.0 as i128),
        );
        token_b_client.transfer(
            &depositor,
            &env.current_contract_address(),
            &(amounts.1 as i128),
        );

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
            depositor,
            new_total_shares - total_shares,
        );
        utils::save_pool_balance_a(&env, balance_a);
        utils::save_pool_balance_b(&env, balance_b);
    }

    fn swap(
        env: Env,
        sender: Address,
        sell_a: bool,
        sell_amount: u128,
        belief_price: Option<u64>,
        max_spread: u64,
    ) {
        sender.require_auth();

        let belief_price = belief_price.map(|val| Decimal::percent(val));
        let max_spread = Decimal::percent(max_spread);

        let pool_balance_a = utils::get_pool_balance_a(&env);
        let pool_balance_b = utils::get_pool_balance_b(&env);
        let (pool_balance_sell, pool_balance_buy) = if sell_a {
            (pool_balance_a, pool_balance_b)
        } else {
            (pool_balance_b, pool_balance_a)
        };

        let (buy_amount, spread_amount, _commission_amount) = compute_swap(
            pool_balance_sell,
            pool_balance_buy,
            sell_amount,
            Decimal::percent(1), // TODO: Add comission rate to the message
        );

        assert_max_spread(
            belief_price,
            max_spread,
            buy_amount,
            sell_amount, /*+ commission_amount*/
            spread_amount,
        );

        let config = get_config(&env);

        // Transfer the amount being sold to the contract
        let (sell_token, buy_token) = if sell_a {
            (config.token_a, config.token_b)
        } else {
            (config.token_b, config.token_a)
        };

        // transfer tokens to swap
        token_contract::Client::new(&env, &sell_token).transfer(
            &sender,
            &env.current_contract_address(),
            &(sell_amount as i128),
        );

        // return swapped tokens to user
        token_contract::Client::new(&env, &buy_token).transfer(
            &env.current_contract_address(),
            &sender,
            &(buy_amount as i128),
        );

        // user is offering to sell A, so they will receive B
        // A balance is bigger, B balance is smaller
        let (balance_a, balance_b) = if sell_a {
            (
                pool_balance_a + sell_amount,
                pool_balance_b /*- protocol_fee_amount */ - buy_amount,
            )
        } else {
            (
                pool_balance_a /*- protocol_fee_amount */ - buy_amount,
                pool_balance_b + sell_amount,
            )
        };
        utils::save_pool_balance_a(&env, balance_a);
        utils::save_pool_balance_b(&env, balance_b);
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

fn assert_slippage_tolerance(
    slippage_tolerance: Option<Decimal>,
    deposits: &[u128; 2],
    pools: &[u128; 2],
) {
    let default_slippage = Decimal::percent(100); // Representing 1.00 (100%) as the default slippage tolerance
    let max_allowed_slippage = Decimal::percent(500); // Representing 5.00 (500%) as the maximum allowed slippage tolerance

    let slippage_tolerance = slippage_tolerance.unwrap_or(default_slippage);
    if slippage_tolerance > max_allowed_slippage {
        panic!("Slippage tolerance exceeds the maximum allowed value");
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

pub fn assert_max_spread(
    belief_price: Option<Decimal>,
    max_spread: Decimal,
    offer_amount: u128,
    return_amount: u128,
    spread_amount: u128,
) {
    let expected_return = belief_price.map(|price| offer_amount * price);

    let total_return = return_amount + spread_amount;

    let spread_ratio = if let Some(expected_return) = expected_return {
        Decimal::from_ratio(spread_amount, expected_return)
    } else {
        Decimal::from_ratio(spread_amount, total_return)
    };

    assert!(spread_ratio <= max_spread, "Spread exceeds maximum allowed");
}

/// Computes the result of a swap operation.
///
/// Arguments:
/// - `offer_pool`: Total amount of offer assets in the pool.
/// - `ask_pool`: Total amount of ask assets in the pool.
/// - `offer_amount`: Amount of offer assets to swap.
/// - `commission_rate`: Total amount of fees charged for the swap.
///
/// Returns a tuple containing the following values:
/// - The resulting amount of ask assets after the swap.
/// - The spread amount, representing the difference between the expected and actual swap amounts.
/// - The commission amount, representing the fees charged for the swap.
pub fn compute_swap(
    offer_pool: u128,
    ask_pool: u128,
    offer_amount: u128,
    commission_rate: Decimal,
) -> (u128, u128, u128) {
    // Calculate the cross product of offer_pool and ask_pool
    let cp: u128 = offer_pool * ask_pool;

    // Calculate the resulting amount of ask assets after the swap
    let return_amount: u128 = ask_pool - (cp / (offer_pool + offer_amount));

    // Calculate the spread amount, representing the difference between the expected and actual swap amounts
    let spread_amount: u128 = (offer_amount * ask_pool / offer_pool) - return_amount;

    let commission_amount: u128 = return_amount * commission_rate;

    // Deduct the commission (minus the part that goes to the protocol) from the return amount
    let return_amount: u128 = return_amount - commission_amount;

    (return_amount, spread_amount, commission_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assert_slippage_tolerance_success() {
        // Test case that should pass:
        // slippage tolerance of 50 (0.5 or 50%), deposits of 10 and 20, pools of 30 and 60
        // The price changes fall within the slippage tolerance
        assert_slippage_tolerance(Some(Decimal::percent(50)), &[10, 20], &[30, 60]);
    }

    #[test]
    #[should_panic(expected = "Slippage tolerance exceeds the maximum allowed value")]
    fn test_assert_slippage_tolerance_fail_tolerance_too_high() {
        // Test case that should fail due to slippage tolerance being too high
        assert_slippage_tolerance(Some(Decimal::percent(600)), &[10, 20], &[30, 60]);
    }

    #[test]
    #[should_panic(expected = "Slippage tolerance violated. Deposits: [10, 15], Pools: [40, 40]")]
    fn test_assert_slippage_tolerance_fail_slippage_violated() {
        // The price changes from 10/15 (0.67) to 40/40 (1.00), violating the 10% slippage tolerance
        assert_slippage_tolerance(Some(Decimal::percent(10)), &[10, 15], &[40, 40]);
    }

    #[test]
    fn test_assert_max_spread_success() {
        // Test case that should pass:
        // belief price of 2.0, max spread of 10%, offer amount of 100k, return amount of 100k and 1 unit, spread amount of 1
        // The spread ratio is 10% which is equal to the max spread
        assert_max_spread(
            Some(Decimal::percent(200)),
            Decimal::percent(10),
            100_000,
            100_001,
            1,
        );
    }

    #[test]
    #[should_panic(expected = "Spread exceeds maximum allowed")]
    fn test_assert_max_spread_fail_max_spread_exceeded() {
        // belief price of 2.0, max spread of 10%, offer amount of 10, return amount of 10, spread amount of 2
        // The spread ratio is 20% which is greater than the max spread
        assert_max_spread(
            Some(Decimal::percent(200)),
            Decimal::percent(10),
            10_000,
            11_000,
            200,
        );
    }

    #[test]
    fn test_assert_max_spread_success_no_belief_price() {
        // no belief price, max spread of 100 (0.1 or 10%), offer amount of 10, return amount of 10, spread amount of 1
        // The spread ratio is 10% which is equal to the max spread
        assert_max_spread(None, Decimal::percent(10), 10, 10, 1);
    }

    // #[test]
    // #[should_panic(expected = "Spread exceeds maximum allowed")]
    // fn test_assert_max_spread_fail_no_belief_price_max_spread_exceeded() {
    //     // no belief price, max spread of 100 (0.1 or 10%), offer amount of 10, return amount of 10, spread amount of 2
    //     // The spread ratio is 20% which is greater than the max spread
    //     assert_max_spread(None, 100, 10, 10, 2);
    // }

    #[test]
    fn test_compute_swap_pass() {
        let result = compute_swap(1000, 2000, 100, Decimal::percent(10)); // 10% commission rate
        assert_eq!(result, (900, 50, 100)); // Expected return amount, spread, and commission
    }

    #[test]
    #[should_panic]
    fn test_compute_swap_panic() {
        let result = compute_swap(1000, 2000, 100, Decimal::one()); // 100% commission rate should lead to panic
        assert_eq!(result, (0, 50, 100)); // This assertion should not matter because of the panic
    }
}
