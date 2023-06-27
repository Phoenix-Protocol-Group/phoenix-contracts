use soroban_sdk::{contractimpl, contractmeta, log, Address, Bytes, BytesN, Env};

use num_integer::Roots;

use crate::{
    error::ContractError,
    storage::{
        get_config, save_config, utils, validate_fee_bps, Asset, Config, PairType, PoolResponse,
        SimulateReverseSwapResponse, SimulateSwapResponse,
    },
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
    #[allow(clippy::too_many_arguments)]
    fn initialize(
        env: Env,
        admin: Address,
        token_wasm_hash: BytesN<32>,
        token_a: Address,
        token_b: Address,
        share_token_decimals: u32,
        swap_fee_bps: i64,
        fee_recipient: Address,
        max_allowed_slippage_bps: i64,
    ) -> Result<(), ContractError>;

    // Deposits token_a and token_b. Also mints pool shares for the "to" Identifier. The amount minted
    // is determined based on the difference between the reserves stored by this contract, and
    // the actual balance of token_a and token_b for this contract.
    fn provide_liquidity(
        env: Env,
        depositor: Address,
        desired_a: i128,
        min_a: Option<i128>,
        desired_b: Option<i128>,
        min_b: Option<i128>,
        custom_slippage_bps: Option<i64>,
    ) -> Result<(), ContractError>;

    // If "buy_a" is true, the swap will buy token_a and sell token_b. This is flipped if "buy_a" is false.
    // "out" is the amount being bought, with in_max being a safety to make sure you receive at least that amount.
    // swap will transfer the selling token "to" to this contract, and then the contract will transfer the buying token to "to".
    fn swap(
        env: Env,
        sender: Address,
        sell_a: bool,
        offer_amount: i128,
        belief_price: Option<i64>,
        max_spread: i64,
    ) -> Result<(), ContractError>;

    // transfers share_amount of pool share tokens to this contract, burns all pools share tokens in this contracts, and sends the
    // corresponding amount of token_a and token_b to "to".
    // Returns amount of both tokens withdrawn
    fn withdraw_liquidity(
        env: Env,
        recipient: Address,
        share_amount: i128,
        min_a: i128,
        min_b: i128,
    ) -> Result<(i128, i128), ContractError>;

    // Migration entrypoint
    fn upgrade(e: Env, new_wasm_hash: BytesN<32>) -> Result<(), ContractError>;

    // QUERIES

    // Returns the configuration structure containing the addresses
    fn query_config(env: Env) -> Result<Config, ContractError>;

    // Returns the address for the pool share token
    fn query_share_token_address(env: Env) -> Result<Address, ContractError>;

    // Returns  the total amount of LP tokens and assets in a specific pool
    fn query_pool_info(env: Env) -> Result<PoolResponse, ContractError>;

    // Simulate swap transaction
    fn simulate_swap(
        env: Env,
        sell_a: bool,
        sell_amount: i128,
    ) -> Result<SimulateSwapResponse, ContractError>;

    // Simulate reverse swap transaction
    fn simulate_reverse_swap(
        env: Env,
        sell_a: bool,
        ask_amount: i128,
    ) -> Result<SimulateReverseSwapResponse, ContractError>;
}

#[contractimpl]
impl LiquidityPoolTrait for LiquidityPool {
    #[allow(clippy::too_many_arguments)]
    fn initialize(
        env: Env,
        admin: Address,
        token_wasm_hash: BytesN<32>,
        token_a: Address,
        token_b: Address,
        share_token_decimals: u32,
        swap_fee_bps: i64,
        fee_recipient: Address,
        max_allowed_slippage_bps: i64,
    ) -> Result<(), ContractError> {
        // Token order validation to make sure only one instance of a pool can exist
        if token_a >= token_b {
            log!(&env, "token_a must be less than token_b");
            return Err(ContractError::FirstTokenMustBeSmallerThenSecond);
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
            token_a: token_a.clone(),
            token_b: token_b.clone(),
            share_token: share_token_address,
            pair_type: PairType::Xyk,
            total_fee_bps: validate_fee_bps(&env, swap_fee_bps)?,
            fee_recipient,
            max_allowed_slippage: max_allowed_slippage_bps,
        };
        save_config(&env, config);
        utils::save_admin(&env, admin);
        utils::save_total_shares(&env, 0);
        utils::save_pool_balance_a(&env, 0);
        utils::save_pool_balance_b(&env, 0);

        env.events()
            .publish(("initialize", "XYK LP token_a"), token_a);
        env.events()
            .publish(("initialize", "XYK LP token_b"), token_b);

        Ok(())
    }

    fn provide_liquidity(
        env: Env,
        sender: Address,
        desired_a: i128,
        min_a: Option<i128>,
        desired_b: Option<i128>,
        min_b: Option<i128>,
        custom_slippage_bps: Option<i64>,
    ) -> Result<(), ContractError> {
        // sender needs to authorize the deposit
        sender.require_auth();

        let pool_balance_a = utils::get_pool_balance_a(&env)?;
        let pool_balance_b = utils::get_pool_balance_b(&env)?;

        let (desired_a, desired_b) = if let Some(desired_b) = desired_b {
            (desired_a, desired_b)
        } else {
            let (a, a_for_swap) =
                divide_provided_deposit(&env, pool_balance_a, pool_balance_b, desired_a, true)?;
            let SimulateSwapResponse {
                ask_amount,
                spread_amount: _,
                commission_amount: _,
                total_return: _,
            } = Self::simulate_swap(env.clone(), true, a_for_swap)?;
            Self::swap(env.clone(), sender.clone(), true, a_for_swap, None, 5)?;

            (a, ask_amount)
        };

        // Calculate deposit amounts
        let amounts = utils::get_deposit_amounts(
            &env,
            desired_a,
            min_a,
            desired_b,
            min_b,
            pool_balance_a,
            pool_balance_b,
        )?;

        let config = get_config(&env)?;

        assert_slippage_tolerance(
            &env,
            custom_slippage_bps,
            &[amounts.0, amounts.1],
            &[pool_balance_a, pool_balance_b],
            config.max_allowed_slippage(),
        )?;

        let token_a_client = token_contract::Client::new(&env, &config.token_a);
        let token_b_client = token_contract::Client::new(&env, &config.token_b);

        // Move tokens from client's wallet to the contract
        token_a_client.transfer(&sender, &env.current_contract_address(), &(amounts.0));
        token_b_client.transfer(&sender, &env.current_contract_address(), &(amounts.1));

        // Now calculate how many new pool shares to mint
        let balance_a = utils::get_balance(&env, &config.token_a);
        let balance_b = utils::get_balance(&env, &config.token_b);
        let total_shares = utils::get_total_shares(&env)?;

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
            &config.share_token,
            &sender,
            new_total_shares - total_shares,
        )?;
        utils::save_pool_balance_a(&env, balance_a);
        utils::save_pool_balance_b(&env, balance_b);

        env.events()
            .publish(("provide_liquidity", "sender"), sender);
        env.events()
            .publish(("provide_liquidity", "token_a"), &config.token_a);
        env.events()
            .publish(("provide_liquidity", "token_a-amount"), amounts.0);
        env.events()
            .publish(("provide_liquidity", "token_a"), &config.token_b);
        env.events()
            .publish(("provide_liquidity", "token_b-amount"), amounts.1);

        Ok(())
    }

    fn swap(
        env: Env,
        sender: Address,
        sell_a: bool,
        offer_amount: i128,
        belief_price: Option<i64>,
        max_spread: i64,
    ) -> Result<(), ContractError> {
        sender.require_auth();

        let belief_price = belief_price.map(Decimal::percent);
        let max_spread = Decimal::percent(max_spread);

        let pool_balance_a = utils::get_pool_balance_a(&env)?;
        let pool_balance_b = utils::get_pool_balance_b(&env)?;
        let (pool_balance_sell, pool_balance_buy) = if sell_a {
            (pool_balance_a, pool_balance_b)
        } else {
            (pool_balance_b, pool_balance_a)
        };

        let config = get_config(&env)?;

        let (return_amount, spread_amount, commission_amount) = compute_swap(
            pool_balance_sell,
            pool_balance_buy,
            offer_amount,
            config.protocol_fee_rate(),
        );

        assert_max_spread(
            &env,
            belief_price,
            max_spread,
            offer_amount,
            return_amount + commission_amount,
            spread_amount,
        )?;

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
            &offer_amount,
        );

        // return swapped tokens to user
        token_contract::Client::new(&env, &buy_token).transfer(
            &env.current_contract_address(),
            &sender,
            &return_amount,
        );

        // send commission to fee recipient
        token_contract::Client::new(&env, &buy_token).transfer(
            &env.current_contract_address(),
            &config.fee_recipient,
            &commission_amount,
        );

        // user is offering to sell A, so they will receive B
        // A balance is bigger, B balance is smaller
        let (balance_a, balance_b) = if sell_a {
            (
                pool_balance_a + offer_amount,
                pool_balance_b - commission_amount - return_amount,
            )
        } else {
            (
                pool_balance_a - commission_amount - return_amount,
                pool_balance_b + offer_amount,
            )
        };
        utils::save_pool_balance_a(&env, balance_a);
        utils::save_pool_balance_b(&env, balance_b);

        env.events().publish(("swap", "sender"), sender);
        env.events().publish(("swap", "sell_token"), sell_token);
        env.events().publish(("swap", "offer_amount"), offer_amount);
        env.events().publish(("swap", "buy_token"), buy_token);
        env.events()
            .publish(("swap", "return_amount"), return_amount);
        env.events()
            .publish(("swap", "spread_amount"), spread_amount);

        Ok(())
    }

    fn withdraw_liquidity(
        env: Env,
        sender: Address,
        share_amount: i128,
        min_a: i128,
        min_b: i128,
    ) -> Result<(i128, i128), ContractError> {
        sender.require_auth();

        let config = get_config(&env)?;

        let share_token_client = token_contract::Client::new(&env, &config.share_token);
        share_token_client.transfer(&sender, &env.current_contract_address(), &share_amount);

        let pool_balance_a = utils::get_pool_balance_a(&env)?;
        let pool_balance_b = utils::get_pool_balance_b(&env)?;

        let mut share_ratio = Decimal::zero();
        let total_shares = utils::get_total_shares(&env)?;
        if total_shares != 0i128 {
            share_ratio = Decimal::from_ratio(share_amount, total_shares);
        }

        let return_amount_a = pool_balance_a * share_ratio;
        let return_amount_b = pool_balance_b * share_ratio;

        if return_amount_a < min_a || return_amount_b < min_b {
            log!(
                &env,
                "Minimum amount of token_a or token_b is not satisfied! min_a: {}, min_b: {}, return_amount_a: {}, return_amount_b: {}",
                min_a,
                min_b,
                return_amount_a,
                return_amount_b
            );
            return Err(ContractError::WithdrawMinNotSatisfied);
        }

        // burn shares
        utils::burn_shares(&env, &config.share_token, share_amount)?;
        // transfer tokens from sender to contract
        token_contract::Client::new(&env, &config.token_a).transfer(
            &env.current_contract_address(),
            &sender,
            &return_amount_a,
        );
        token_contract::Client::new(&env, &config.token_b).transfer(
            &env.current_contract_address(),
            &sender,
            &return_amount_b,
        );
        // update pool balances
        utils::save_pool_balance_a(&env, pool_balance_a - return_amount_a);
        utils::save_pool_balance_b(&env, pool_balance_b - return_amount_b);

        env.events()
            .publish(("withdraw_liquidity", "sender"), sender);
        env.events()
            .publish(("withdraw_liquidity", "shares_amount"), share_amount);
        env.events()
            .publish(("withdraw_liquidity", "return_amount_a"), return_amount_a);
        env.events()
            .publish(("withdraw_liquidity", "return_amount_b"), return_amount_b);

        Ok((return_amount_a, return_amount_b))
    }

    fn upgrade(env: Env, new_wasm_hash: BytesN<32>) -> Result<(), ContractError> {
        let admin: Address = utils::get_admin(&env)?;
        admin.require_auth();

        env.update_current_contract_wasm(&new_wasm_hash);
        Ok(())
    }

    // Queries

    fn query_config(env: Env) -> Result<Config, ContractError> {
        get_config(&env)
    }

    fn query_share_token_address(env: Env) -> Result<Address, ContractError> {
        Ok(get_config(&env)?.share_token)
    }

    fn query_pool_info(env: Env) -> Result<PoolResponse, ContractError> {
        let config = get_config(&env)?;

        Ok(PoolResponse {
            asset_a: Asset {
                address: config.token_a,
                amount: utils::get_pool_balance_a(&env)?,
            },
            asset_b: Asset {
                address: config.token_b,
                amount: utils::get_pool_balance_b(&env)?,
            },
            asset_lp_share: Asset {
                address: config.share_token,
                amount: utils::get_total_shares(&env)?,
            },
        })
    }

    fn simulate_swap(
        env: Env,
        sell_a: bool,
        offer_amount: i128,
    ) -> Result<SimulateSwapResponse, ContractError> {
        let pool_balance_a = utils::get_pool_balance_a(&env)?;
        let pool_balance_b = utils::get_pool_balance_b(&env)?;
        let (pool_balance_offer, pool_balance_ask) = if sell_a {
            (pool_balance_a, pool_balance_b)
        } else {
            (pool_balance_b, pool_balance_a)
        };

        let config = get_config(&env)?;

        let (ask_amount, spread_amount, commission_amount) = compute_swap(
            pool_balance_offer,
            pool_balance_ask,
            offer_amount,
            config.protocol_fee_rate(),
        );

        let total_return = ask_amount + commission_amount + spread_amount;

        Ok(SimulateSwapResponse {
            ask_amount,
            spread_amount,
            commission_amount,
            total_return,
        })
    }

    fn simulate_reverse_swap(
        env: Env,
        sell_a: bool,
        ask_amount: i128,
    ) -> Result<SimulateReverseSwapResponse, ContractError> {
        let pool_balance_a = utils::get_pool_balance_a(&env)?;
        let pool_balance_b = utils::get_pool_balance_b(&env)?;
        let (pool_balance_offer, pool_balance_ask) = if sell_a {
            (pool_balance_a, pool_balance_b)
        } else {
            (pool_balance_b, pool_balance_a)
        };

        let config = get_config(&env)?;

        let (offer_amount, spread_amount, commission_amount) = compute_offer_amount(
            pool_balance_offer,
            pool_balance_ask,
            ask_amount,
            config.protocol_fee_rate(),
        )?;

        Ok(SimulateReverseSwapResponse {
            offer_amount,
            spread_amount,
            commission_amount,
        })
    }
}

/// Divides `deposit` into parts to maintain the pool ratio.
/// Returns the amount of A and B tokens to add to the pool.
///
/// * **a_pool** current amount of A tokens in the pool.
/// * **b_pool** current deposit of B tokens in the pool.
/// * **deposit** total deposit of tokens to provide.
fn divide_provided_deposit(
    env: &Env,
    a_pool: i128,
    b_pool: i128,
    deposit: i128,
    sell_a: bool,
) -> Result<(i128, i128), ContractError> {
    // Validate the inputs
    if a_pool <= 0 || b_pool <= 0 || deposit <= 0 {
        log!(env, "Both pools and deposit must be a positive!");
        return Err(ContractError::EmptyPoolBalance);
    }

    // Calculate the current ratio in the pool
    let ratio = Decimal::from_ratio(b_pool, a_pool);

    let (a_to_add, b_to_add) = if sell_a {
        // Solve the system of equations: a + b = deposit and b/a = ratio
        let a = deposit * (Decimal::one() + ratio).inv().unwrap();
        let b = ratio * a;
        (a, b)
    } else {
        // Solve the system of equations: a + b = deposit and a/b = ratio
        let b = deposit * (Decimal::one() + Decimal::one() / ratio).inv().unwrap();
        let a = b * ratio.inv().unwrap();
        (b, a)
    };

    Ok((a_to_add, b_to_add))
}

fn assert_slippage_tolerance(
    env: &Env,
    slippage_tolerance: Option<i64>,
    deposits: &[i128; 2],
    pools: &[i128; 2],
    max_allowed_slippage: Decimal,
) -> Result<(), ContractError> {
    let default_slippage = Decimal::percent(1); // Representing 1% as the default slippage tolerance

    let slippage_tolerance = if let Some(slippage_tolerance) = slippage_tolerance {
        Decimal::bps(slippage_tolerance)
    } else {
        default_slippage
    };
    if slippage_tolerance > max_allowed_slippage {
        log!(env, "Slippage tolerance exceeds the maximum allowed value");
        return Err(ContractError::SlippageToleranceExceeded);
    }

    let slippage_tolerance = slippage_tolerance * 100; // Converting to a percentage value
    let one_minus_slippage_tolerance = 10000 - slippage_tolerance;
    let deposits: [i128; 2] = [deposits[0], deposits[1]];
    let pools: [i128; 2] = [pools[0], pools[1]];

    // Ensure each price does not change more than what the slippage tolerance allows
    if deposits[0] * pools[1] * one_minus_slippage_tolerance > deposits[1] * pools[0] * 10000
        || deposits[1] * pools[0] * one_minus_slippage_tolerance > deposits[0] * pools[1] * 10000
    {
        log!(
            env,
            "Slippage tolerance violated. Deposits: 0: {} 1: {}, Pools: 0: {} 1: {}",
            deposits[0],
            deposits[1],
            pools[0],
            pools[1]
        );
        return Err(ContractError::SlippageToleranceViolated);
    }
    Ok(())
}

pub fn assert_max_spread(
    env: &Env,
    belief_price: Option<Decimal>,
    max_spread: Decimal,
    offer_amount: i128,
    return_amount: i128,
    spread_amount: i128,
) -> Result<(), ContractError> {
    let expected_return = belief_price.map(|price| offer_amount * price);

    let total_return = return_amount + spread_amount;

    let spread_ratio = if let Some(expected_return) = expected_return {
        Decimal::from_ratio(spread_amount, expected_return)
    } else {
        Decimal::from_ratio(spread_amount, total_return)
    };

    if spread_ratio > max_spread {
        log!(env, "Spread exceeds maximum allowed");
        return Err(ContractError::SpreadExceedsMaxAllowed);
    }
    Ok(())
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
    offer_pool: i128,
    ask_pool: i128,
    offer_amount: i128,
    commission_rate: Decimal,
) -> (i128, i128, i128) {
    // Calculate the cross product of offer_pool and ask_pool
    let cp: i128 = offer_pool * ask_pool;

    // Calculate the resulting amount of ask assets after the swap
    let return_amount: i128 = ask_pool - (cp / (offer_pool + offer_amount));

    // Calculate the spread amount, representing the difference between the expected and actual swap amounts
    let spread_amount: i128 = (offer_amount * ask_pool / offer_pool) - return_amount;

    let commission_amount: i128 = return_amount * commission_rate;

    // Deduct the commission (minus the part that goes to the protocol) from the return amount
    let return_amount: i128 = return_amount - commission_amount;

    (return_amount, spread_amount, commission_amount)
}

/// Returns an amount of offer assets for a specified amount of ask assets.
///
/// * **offer_pool** total amount of offer assets in the pool.
/// * **ask_pool** total amount of ask assets in the pool.
/// * **ask_amount** amount of ask assets to swap to.
/// * **commission_rate** total amount of fees charged for the swap.
pub fn compute_offer_amount(
    offer_pool: i128,
    ask_pool: i128,
    ask_amount: i128,
    commission_rate: Decimal,
) -> Result<(i128, i128, i128), ContractError> {
    // Calculate the cross product of offer_pool and ask_pool
    let cp: i128 = offer_pool * ask_pool;

    // Calculate one minus the commission rate
    let one_minus_commission = Decimal::one() - commission_rate;

    // Calculate the inverse of one minus the commission rate
    let inv_one_minus_commission = Decimal::one() / one_minus_commission;

    // Calculate the resulting amount of ask assets after the swap
    let offer_amount: i128 = cp / (ask_pool - (ask_amount * inv_one_minus_commission)) - offer_pool;

    let ask_before_commission = ask_amount * inv_one_minus_commission;

    // Calculate the spread amount, representing the difference between the expected and actual swap amounts
    let spread_amount: i128 = (offer_amount * ask_pool / offer_pool) - ask_before_commission;

    // Calculate the commission amount
    let commission_amount: i128 = ask_before_commission * commission_rate;

    Ok((offer_amount, spread_amount, commission_amount))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assert_slippage_tolerance_success() {
        let env = Env::default();
        // Test case that should pass:
        // slippage tolerance of 5000 (0.5 or 50%), deposits of 10 and 20, pools of 30 and 60
        // The price changes fall within the slippage tolerance
        let max_allowed_slippage = 5_000i64;
        assert_slippage_tolerance(
            &env,
            Some(max_allowed_slippage),
            &[10, 20],
            &[30, 60],
            Decimal::bps(max_allowed_slippage),
        )
        .unwrap();
    }

    #[test]
    fn test_assert_slippage_tolerance_fail_tolerance_too_high() {
        let env = Env::default();
        // Test case that should fail due to slippage tolerance being too high
        let max_allowed_slippage = Decimal::bps(5_000i64);
        let result = assert_slippage_tolerance(
            &env,
            Some(60_000),
            &[10, 20],
            &[30, 60],
            max_allowed_slippage,
        )
        .unwrap_err();
        assert_eq!(ContractError::SlippageToleranceExceeded, result);
    }

    #[test]
    fn test_assert_slippage_tolerance_fail_slippage_violated() {
        let env = Env::default();
        let max_allowed_slippage = Decimal::bps(5_000i64);
        // The price changes from 10/15 (0.67) to 40/40 (1.00), violating the 10% slippage tolerance
        let result = assert_slippage_tolerance(
            &env,
            Some(1_000),
            &[10, 15],
            &[40, 40],
            max_allowed_slippage,
        )
        .unwrap_err();
        assert_eq!(ContractError::SlippageToleranceViolated, result);
    }

    #[test]
    fn test_assert_max_spread_success() {
        let env = Env::default();
        // Test case that should pass:
        // belief price of 2.0, max spread of 10%, offer amount of 100k, return amount of 100k and 1 unit, spread amount of 1
        // The spread ratio is 10% which is equal to the max spread
        assert_max_spread(
            &env,
            Some(Decimal::percent(200)),
            Decimal::percent(10),
            100_000,
            100_001,
            1,
        )
        .unwrap();
    }

    #[test]
    fn test_assert_max_spread_fail_max_spread_exceeded() {
        let env = Env::default();

        let belief_price = Some(Decimal::percent(250)); // belief price is 2.5
        let max_spread = Decimal::percent(10); // 10% is the maximum allowed spread
        let offer_amount = 100;
        let return_amount = 100; // These values are chosen such that the spread ratio will be more than 10%
        let spread_amount = 35;

        let result = assert_max_spread(
            &env,
            belief_price,
            max_spread,
            offer_amount,
            return_amount,
            spread_amount,
        )
        .unwrap_err();
        assert_eq!(ContractError::SpreadExceedsMaxAllowed, result);
    }

    #[test]
    fn test_assert_max_spread_success_no_belief_price() {
        let env = Env::default();
        // no belief price, max spread of 100 (0.1 or 10%), offer amount of 10, return amount of 10, spread amount of 1
        // The spread ratio is 10% which is equal to the max spread
        assert_max_spread(&env, None, Decimal::percent(10), 10, 10, 1).unwrap();
    }

    #[test]
    fn test_assert_max_spread_fail_no_belief_price_max_spread_exceeded() {
        let env = Env::default();
        // no belief price, max spread of 10%, offer amount of 10, return amount of 10, spread amount of 2
        // The spread ratio is 20% which is greater than the max spread
        let result = assert_max_spread(&env, None, Decimal::percent(10), 10, 10, 2).unwrap_err();
        assert_eq!(ContractError::SpreadExceedsMaxAllowed, result);
    }

    #[test]
    fn test_compute_swap_pass() {
        let result = compute_swap(1000, 2000, 100, Decimal::percent(10)); // 10% commission rate
        assert_eq!(result, (164, 18, 18)); // Expected return amount, spread, and commission
    }

    #[test]
    fn test_compute_swap_full_commission() {
        let result = compute_swap(1000, 2000, 100, Decimal::one()); // 100% commission rate should lead to return_amount being 0
        assert_eq!(result, (0, 18, 182));
    }

    #[test]
    fn test_compute_offer_amount() {
        let offer_pool = 1000000;
        let ask_pool = 1000000;
        let commission_rate = Decimal::percent(10);
        let ask_amount = 1000;

        let result =
            compute_offer_amount(offer_pool, ask_pool, ask_amount, commission_rate).unwrap();

        // Test that the offer amount is less than the original pool size, due to commission
        assert!(result.0 < offer_pool);

        // Test that the spread amount is non-negative
        assert!(result.1 >= 0);

        // Test that the commission amount is exactly 10% of the offer amount
        assert_eq!(result.2, result.0 * Decimal::percent(10));
    }
}
