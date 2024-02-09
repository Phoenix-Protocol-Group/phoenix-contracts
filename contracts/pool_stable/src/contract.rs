use phoenix::utils::LiquidityPoolInitInfo;
use soroban_sdk::{contract, contractimpl, contractmeta, log, Address, BytesN, Env, IntoVal};

use crate::storage::utils::{is_initialized, set_initialized};
use crate::storage::StableLiquidityPoolInfo;
use crate::{
    math::{calc_y, compute_current_amp, compute_d, AMP_PRECISION},
    stake_contract,
    storage::{
        get_amp, get_config, get_greatest_precision, save_amp, save_config,
        save_greatest_precision, utils, validate_fee_bps, AmplifierParameters, Asset, Config,
        PairType, PoolResponse, SimulateReverseSwapResponse, SimulateSwapResponse,
    },
    token_contract,
};
use decimal::Decimal;
use phoenix::{validate_bps, validate_int_parameters};

// Minimum amount of initial LP shares to mint
const MINIMUM_LIQUIDITY_AMOUNT: i128 = 1000;

// Metadata that is added on to the WASM custom section
contractmeta!(
    key = "Description",
    val = "Phoenix Protocol Stable Liquidity Pool"
);

#[contract]
pub struct StableLiquidityPool;

pub trait StableLiquidityPoolTrait {
    // Sets the token contract addresses for this pool
    // token_wasm_hash is the WASM hash of the deployed token contract for the pool share token
    #[allow(clippy::too_many_arguments)]
    fn initialize(
        env: Env,
        stake_wasm_hash: BytesN<32>,
        token_wasm_hash: BytesN<32>,
        amp: u64,
        lp_init_info: LiquidityPoolInitInfo,
    );

    // Deposits token_a and token_b. Also mints pool shares for the "to" Identifier. The amount minted
    // is determined based on the difference between the reserves stored by this contract, and
    // the actual balance of token_a and token_b for this contract.
    fn provide_liquidity(
        env: Env,
        depositor: Address,
        desired_a: i128,
        desired_b: i128,
        custom_slippage_bps: Option<i64>,
    );

    // `offer_asset` is the asset that the user would like to swap for the other token in the pool.
    // `offer_amount` is the amount being sold, with `max_spread_bps` being a safety to make sure you receive at least that amount.
    // swap will transfer the selling token "to" to this contract, and then the contract will transfer the buying token to `sender`.
    // Returns the amount of the token being bought.
    fn swap(
        env: Env,
        sender: Address,
        offer_asset: Address,
        offer_amount: i128,
        belief_price: Option<i64>,
        max_spread_bps: Option<i64>,
    ) -> i128;

    // transfers share_amount of pool share tokens to this contract, burns all pools share tokens in this contracts, and sends the
    // corresponding amount of token_a and token_b to "to".
    // Returns amount of both tokens withdrawn
    fn withdraw_liquidity(
        env: Env,
        recipient: Address,
        share_amount: i128,
        min_a: i128,
        min_b: i128,
    ) -> (i128, i128);

    // Allows admin address set during initialization to change some parameters of the
    // configuration
    fn update_config(
        env: Env,
        sender: Address,
        new_admin: Option<Address>,
        total_fee_bps: Option<i64>,
        fee_recipient: Option<Address>,
        max_allowed_slippage_bps: Option<i64>,
        max_allowed_spread_bps: Option<i64>,
    );

    // Migration entrypoint
    fn upgrade(e: Env, new_wasm_hash: BytesN<32>);

    // QUERIES

    // Returns the configuration structure containing the addresses
    fn query_config(env: Env) -> Config;

    // Returns the address for the pool share token
    fn query_share_token_address(env: Env) -> Address;

    // Returns the address for the pool stake contract
    fn query_stake_contract_address(env: Env) -> Address;

    // Returns  the total amount of LP tokens and assets in a specific pool
    fn query_pool_info(env: Env) -> PoolResponse;

    fn query_pool_info_for_factory(env: Env) -> StableLiquidityPoolInfo;

    // Simulate swap transaction
    fn simulate_swap(env: Env, offer_asset: Address, sell_amount: i128) -> SimulateSwapResponse;

    // Simulate reverse swap transaction
    fn simulate_reverse_swap(
        env: Env,
        offer_asset: Address,
        ask_amount: i128,
    ) -> SimulateReverseSwapResponse;
}

#[contractimpl]
impl StableLiquidityPoolTrait for StableLiquidityPool {
    #[allow(clippy::too_many_arguments)]
    fn initialize(
        env: Env,
        stake_wasm_hash: BytesN<32>,
        token_wasm_hash: BytesN<32>,
        amp: u64,
        lp_init_info: LiquidityPoolInitInfo,
    ) {
        if is_initialized(&env) {
            panic!("Pool stable: Initialize: initializing contract twice is not allowed");
        }

        let admin = lp_init_info.admin;
        let share_token_decimals = lp_init_info.share_token_decimals;
        let swap_fee_bps = lp_init_info.swap_fee_bps;
        let fee_recipient = lp_init_info.fee_recipient;
        let max_allowed_slippage_bps = lp_init_info.max_allowed_slippage_bps;
        let max_allowed_spread_bps = lp_init_info.max_allowed_spread_bps;
        let token_init_info = lp_init_info.token_init_info;
        let stake_init_info = lp_init_info.stake_init_info;

        validate_bps!(
            swap_fee_bps,
            max_allowed_slippage_bps,
            max_allowed_spread_bps
        );
        set_initialized(&env);

        // Token info
        let token_a = token_init_info.token_a;
        let token_b = token_init_info.token_b;
        // Contract info
        let min_bond = stake_init_info.min_bond;
        let max_distributions = stake_init_info.max_distributions;
        let min_reward = stake_init_info.min_reward;

        // Token order validation to make sure only one instance of a pool can exist
        if token_a >= token_b {
            log!(&env, "token_a must be less than token_b");
            panic!(
                "Pool: Initialize: First token must be alphabetically smaller than second token"
            );
        }

        save_greatest_precision(&env, &token_a, &token_b);

        if !(0..=10_000).contains(&swap_fee_bps) {
            log!(&env, "Fees must be between 0 and 100%");
            panic!("Pool: Initialize: Fees must be between 0 and 100%");
        }

        // deploy token contract
        let share_token_address =
            utils::deploy_token_contract(&env, token_wasm_hash, &token_a, &token_b);
        token_contract::Client::new(&env, &share_token_address).initialize(
            // admin
            &env.current_contract_address(),
            // number of decimals on the share token
            &share_token_decimals,
            // name
            &"Pool Share Token".into_val(&env),
            // symbol
            &"POOL".into_val(&env),
        );

        let stake_contract_address = utils::deploy_stake_contract(&env, stake_wasm_hash);
        stake_contract::Client::new(&env, &stake_contract_address).initialize(
            &admin,
            &share_token_address,
            &min_bond,
            &max_distributions,
            &min_reward,
        );

        let config = Config {
            token_a: token_a.clone(),
            token_b: token_b.clone(),
            share_token: share_token_address,
            stake_contract: stake_contract_address,
            pool_type: PairType::Xyk,
            total_fee_bps: validate_fee_bps(&env, swap_fee_bps),
            fee_recipient,
            max_allowed_slippage_bps,
            max_allowed_spread_bps,
        };
        save_config(&env, config);
        let current_time = env.ledger().timestamp();
        save_amp(
            &env,
            AmplifierParameters {
                init_amp: amp * AMP_PRECISION,
                init_amp_time: current_time,
                next_amp: amp * AMP_PRECISION,
                next_amp_time: current_time,
            },
        );
        utils::save_admin(&env, admin);
        utils::save_total_shares(&env, 0);
        utils::save_pool_balance_a(&env, 0);
        utils::save_pool_balance_b(&env, 0);

        env.events()
            .publish(("initialize", "XYK LP token_a"), token_a);
        env.events()
            .publish(("initialize", "XYK LP token_b"), token_b);
    }

    fn provide_liquidity(
        env: Env,
        sender: Address,
        desired_a: i128,
        desired_b: i128,
        custom_slippage_bps: Option<i64>,
    ) {
        validate_int_parameters!(desired_a, desired_b);

        // sender needs to authorize the deposit
        sender.require_auth();

        let config = get_config(&env);
        let greatest_precision = get_greatest_precision(&env);
        let old_balance_a = utils::get_pool_balance_a(&env);
        let old_balance_b = utils::get_pool_balance_b(&env);

        // Check if custom_slippage_bps is more than max_allowed_slippage
        if let Some(custom_slippage) = custom_slippage_bps {
            if custom_slippage > config.max_allowed_slippage_bps {
                panic!("Pool: ProvideLiquidity: Custom slippage tolerance is more than max allowed slippage tolerance");
            }
        }

        let amp_parameters = get_amp(&env).unwrap(); // FIXME: This is minor, but add some
                                                     // validation to AMP parameters
        let amp = compute_current_amp(&env, &amp_parameters);

        // Invariant (D) after deposit added
        let new_balance_a = desired_a + old_balance_a;
        let new_balance_b = desired_b + old_balance_b;
        let new_invariant = compute_d(
            amp as u128,
            &[
                Decimal::from_atomics(new_balance_a, 6),
                Decimal::from_atomics(new_balance_b, 6),
            ],
        );

        let total_shares = utils::get_total_shares(&env);
        let shares = if total_shares == 0 {
            let share =
                new_invariant.to_i128_with_precision(greatest_precision) - MINIMUM_LIQUIDITY_AMOUNT;
            if share == 0 {
                panic!("Pool: ProvideLiquidity: Liquidity amount is too low");
            }
            share
        } else {
            let initial_invariant = compute_d(
                amp as u128,
                &[
                    Decimal::from_atomics(old_balance_a, 6),
                    Decimal::from_atomics(old_balance_b, 6),
                ],
            );
            // Calculate the proportion of the change in invariant
            (Decimal::from_ratio((new_invariant - initial_invariant) * total_shares, 1)
                / initial_invariant)
                .to_i128_with_precision(greatest_precision)
        };

        let token_a_client = token_contract::Client::new(&env, &config.token_a);
        let token_b_client = token_contract::Client::new(&env, &config.token_b);

        // Move tokens from client's wallet to the contract
        token_a_client.transfer(&sender, &env.current_contract_address(), &(desired_a));
        token_b_client.transfer(&sender, &env.current_contract_address(), &(desired_b));

        // Now calculate how many new pool shares to mint
        let balance_a = utils::get_balance(&env, &config.token_a);
        let balance_b = utils::get_balance(&env, &config.token_b);

        utils::mint_shares(&env, &config.share_token, &sender, shares);
        utils::save_pool_balance_a(&env, balance_a);
        utils::save_pool_balance_b(&env, balance_b);

        env.events()
            .publish(("provide_liquidity", "sender"), sender);
        env.events()
            .publish(("provide_liquidity", "token_a"), &config.token_a);
        env.events()
            .publish(("provide_liquidity", "token_a-amount"), desired_a);
        env.events()
            .publish(("provide_liquidity", "token_b"), &config.token_b);
        env.events()
            .publish(("provide_liquidity", "token_b-amount"), desired_b);
    }

    fn swap(
        env: Env,
        sender: Address,
        offer_asset: Address,
        offer_amount: i128,
        belief_price: Option<i64>,
        max_spread_bps: Option<i64>,
    ) -> i128 {
        validate_int_parameters!(offer_amount);

        sender.require_auth();

        do_swap(
            env,
            sender,
            offer_asset,
            offer_amount,
            belief_price,
            max_spread_bps,
        )
    }

    fn withdraw_liquidity(
        env: Env,
        sender: Address,
        share_amount: i128,
        min_a: i128,
        min_b: i128,
    ) -> (i128, i128) {
        validate_int_parameters!(share_amount, min_a, min_b);

        sender.require_auth();

        let config = get_config(&env);

        let share_token_client = token_contract::Client::new(&env, &config.share_token);
        share_token_client.transfer(&sender, &env.current_contract_address(), &share_amount);

        let pool_balance_a = utils::get_pool_balance_a(&env);
        let pool_balance_b = utils::get_pool_balance_b(&env);

        let mut share_ratio = Decimal::zero();
        let total_shares = utils::get_total_shares(&env);
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
            panic!(
                "Pool: WithdrawLiquidity: Minimum amount of token_a or token_b is not satisfied!"
            )
        }

        // burn shares
        utils::burn_shares(&env, &config.share_token, share_amount);
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

        (return_amount_a, return_amount_b)
    }

    fn update_config(
        env: Env,
        sender: Address,
        new_admin: Option<Address>,
        total_fee_bps: Option<i64>,
        fee_recipient: Option<Address>,
        max_allowed_slippage_bps: Option<i64>,
        max_allowed_spread_bps: Option<i64>,
    ) {
        if sender != utils::get_admin(&env) {
            panic!("Pool: UpdateConfig: Unauthorized");
        }

        let mut config = get_config(&env);

        if let Some(new_admin) = new_admin {
            utils::save_admin(&env, new_admin);
        }
        if let Some(total_fee_bps) = total_fee_bps {
            if !(0..=10_000).contains(&total_fee_bps) {
                panic!("Pool: UpdateConfig: Invalid total_fee_bps");
            }
            config.total_fee_bps = total_fee_bps;
        }
        if let Some(fee_recipient) = fee_recipient {
            config.fee_recipient = fee_recipient;
        }
        if let Some(max_allowed_slippage_bps) = max_allowed_slippage_bps {
            config.max_allowed_slippage_bps = max_allowed_slippage_bps;
        }
        if let Some(max_allowed_spread_bps) = max_allowed_spread_bps {
            config.max_allowed_spread_bps = max_allowed_spread_bps;
        }

        save_config(&env, config);
    }

    fn upgrade(env: Env, new_wasm_hash: BytesN<32>) {
        let admin: Address = utils::get_admin(&env);
        admin.require_auth();

        env.deployer().update_current_contract_wasm(new_wasm_hash);
    }

    // Queries

    fn query_config(env: Env) -> Config {
        get_config(&env)
    }

    fn query_share_token_address(env: Env) -> Address {
        get_config(&env).share_token
    }

    fn query_stake_contract_address(env: Env) -> Address {
        get_config(&env).stake_contract
    }

    fn query_pool_info(env: Env) -> PoolResponse {
        let config = get_config(&env);

        PoolResponse {
            asset_a: Asset {
                address: config.token_a,
                amount: utils::get_pool_balance_a(&env),
            },
            asset_b: Asset {
                address: config.token_b,
                amount: utils::get_pool_balance_b(&env),
            },
            asset_lp_share: Asset {
                address: config.share_token,
                amount: utils::get_total_shares(&env),
            },
        }
    }

    fn query_pool_info_for_factory(env: Env) -> StableLiquidityPoolInfo {
        let config = get_config(&env);
        let pool_response = PoolResponse {
            asset_a: Asset {
                address: config.token_a,
                amount: utils::get_pool_balance_a(&env),
            },
            asset_b: Asset {
                address: config.token_b,
                amount: utils::get_pool_balance_b(&env),
            },
            asset_lp_share: Asset {
                address: config.share_token,
                amount: utils::get_total_shares(&env),
            },
        };
        let total_fee_bps = config.total_fee_bps;

        StableLiquidityPoolInfo {
            pool_address: env.current_contract_address(),
            pool_response,
            total_fee_bps,
        }
    }

    fn simulate_swap(env: Env, offer_asset: Address, offer_amount: i128) -> SimulateSwapResponse {
        let config = get_config(&env);

        let pool_balance_a = utils::get_pool_balance_a(&env);
        let pool_balance_b = utils::get_pool_balance_b(&env);
        let (pool_balance_offer, pool_balance_ask) = if offer_asset == config.token_a {
            (pool_balance_a, pool_balance_b)
        } else {
            (pool_balance_b, pool_balance_a)
        };

        let (ask_amount, spread_amount, commission_amount) = compute_swap(
            &env,
            pool_balance_offer,
            pool_balance_ask,
            offer_amount,
            config.protocol_fee_rate(),
        );

        let total_return = ask_amount + commission_amount + spread_amount;

        SimulateSwapResponse {
            ask_amount,
            spread_amount,
            commission_amount,
            total_return,
        }
    }

    fn simulate_reverse_swap(
        env: Env,
        offer_asset: Address,
        ask_amount: i128,
    ) -> SimulateReverseSwapResponse {
        let config = get_config(&env);

        let pool_balance_a = utils::get_pool_balance_a(&env);
        let pool_balance_b = utils::get_pool_balance_b(&env);
        let (pool_balance_offer, pool_balance_ask) = if offer_asset == config.token_a {
            (pool_balance_a, pool_balance_b)
        } else {
            (pool_balance_b, pool_balance_a)
        };

        let (offer_amount, spread_amount, commission_amount) = compute_offer_amount(
            &env,
            pool_balance_offer,
            pool_balance_ask,
            ask_amount,
            config.protocol_fee_rate(),
        );

        SimulateReverseSwapResponse {
            offer_amount,
            spread_amount,
            commission_amount,
        }
    }
}

fn do_swap(
    env: Env,
    sender: Address,
    offer_asset: Address,
    offer_amount: i128,
    belief_price: Option<i64>,
    max_spread: Option<i64>,
) -> i128 {
    let config = get_config(&env);

    let belief_price = belief_price.map(Decimal::percent);
    let max_spread = Decimal::bps(max_spread.map_or_else(|| config.max_allowed_spread_bps, |x| x));

    let pool_balance_a = utils::get_pool_balance_a(&env);
    let pool_balance_b = utils::get_pool_balance_b(&env);

    let (pool_balance_sell, pool_balance_buy) = if offer_asset == config.token_a {
        (pool_balance_a, pool_balance_b)
    } else {
        (pool_balance_b, pool_balance_a)
    };

    let (return_amount, spread_amount, commission_amount) = compute_swap(
        &env,
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
    );

    // Transfer the amount being sold to the contract
    let (sell_token, buy_token) = if offer_asset == config.clone().token_a {
        (config.clone().token_a, config.clone().token_b)
    } else {
        (config.clone().token_b, config.clone().token_a)
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
    let (balance_a, balance_b) = if offer_asset == config.token_a {
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

    return_amount
}

/// This function asserts that the spread (slippage) does not exceed a given maximum.
/// * `belief_price` - An optional user-provided belief price, i.e., the expected price per token.
/// * `max_spread` - The maximum allowed spread (slippage) as a fraction of the return amount.
/// * `offer_amount` - The amount of tokens that the user offers to swap.
/// * `return_amount` - The amount of tokens that the user receives in return.
/// * `spread_amount` - The spread (slippage) amount, i.e., the difference between the expected and actual return.
/// # Returns
/// * An error if the spread exceeds the maximum allowed, otherwise Ok.
pub fn assert_max_spread(
    env: &Env,
    belief_price: Option<Decimal>,
    max_spread: Decimal,
    offer_amount: i128,
    return_amount: i128,
    spread_amount: i128,
) {
    // Calculate the expected return if a belief price is provided
    let expected_return = belief_price.map(|price| offer_amount * price);

    // Total return is the sum of the amount received and the spread
    let total_return = return_amount + spread_amount;

    // Calculate the spread ratio, the fraction of the return that is due to spread
    // If the user has specified a belief price, use it to calculate the expected return
    // Otherwise, use the total return
    let spread_ratio = if let Some(expected_return) = expected_return {
        Decimal::from_ratio(spread_amount, expected_return)
    } else {
        Decimal::from_ratio(spread_amount, total_return)
    };

    if spread_ratio > max_spread {
        log!(env, "Spread exceeds maximum allowed");
        panic!("Pool: Assert max spread: spread exceeds maximum allowed");
    }
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
/// - The resulting amount of ask assets after the swap minus the commission amount.
/// - The spread amount, representing the difference between the expected and actual swap amounts.
/// - The commission amount, representing the fees charged for the swap.
pub fn compute_swap(
    env: &Env,
    offer_pool: i128,
    ask_pool: i128,
    offer_amount: i128,
    commission_rate: Decimal,
) -> (i128, i128, i128) {
    let amp_parameters = get_amp(env).unwrap();
    let amp = compute_current_amp(env, &amp_parameters);

    let new_ask_pool = calc_y(
        amp as u128,
        Decimal::from_atomics(offer_pool + offer_amount, 6),
        &[
            Decimal::from_atomics(offer_pool, 6),
            Decimal::from_atomics(ask_pool, 6),
        ],
        6,
    );

    let return_amount = ask_pool - new_ask_pool;
    // We consider swap rate 1:1 in stable swap thus any difference is considered as spread.
    let spread_amount = offer_amount - return_amount;
    let commission_amount = return_amount * commission_rate;
    // Because of issue #211
    let return_amount = return_amount - commission_amount;

    (return_amount, spread_amount, commission_amount)
}

/// Returns an amount of offer assets for a specified amount of ask assets.
///
/// * **offer_pool** total amount of offer assets in the pool.
/// * **ask_pool** total amount of ask assets in the pool.
/// * **ask_amount** amount of ask assets to swap to.
/// * **commission_rate** total amount of fees charged for the swap.
pub fn compute_offer_amount(
    env: &Env,
    offer_pool: i128,
    ask_pool: i128,
    ask_amount: i128,
    commission_rate: Decimal,
) -> (i128, i128, i128) {
    let amp_parameters = get_amp(env).unwrap();
    let amp = compute_current_amp(env, &amp_parameters);

    let new_offer_pool = calc_y(
        amp as u128,
        Decimal::from_atomics(ask_pool - ask_amount, 6),
        &[
            Decimal::from_atomics(offer_pool, 6),
            Decimal::from_atomics(ask_pool, 6),
        ],
        6,
    );

    let offer_amount = new_offer_pool - offer_pool;

    let one_minus_commission = Decimal::one() - commission_rate;
    let inv_one_minus_commission = Decimal::one() / one_minus_commission;
    let ask_before_commission = ask_amount * inv_one_minus_commission;
    // Calculate the spread amount, representing the difference between the expected and actual swap amounts
    let spread_amount: i128 = (offer_amount * ask_pool / offer_pool) - ask_before_commission;

    // Calculate the commission amount
    let commission_amount: i128 = ask_before_commission * commission_rate;

    (offer_amount, spread_amount, commission_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

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
        );
    }

    #[test]
    #[should_panic(expected = "spread exceeds maximum allowed")]
    fn test_assert_max_spread_fail_max_spread_exceeded() {
        let env = Env::default();

        let belief_price = Some(Decimal::percent(250)); // belief price is 2.5
        let max_spread = Decimal::percent(10); // 10% is the maximum allowed spread
        let offer_amount = 100;
        let return_amount = 100; // These values are chosen such that the spread ratio will be more than 10%
        let spread_amount = 35;

        assert_max_spread(
            &env,
            belief_price,
            max_spread,
            offer_amount,
            return_amount,
            spread_amount,
        );
    }

    #[test]
    fn test_assert_max_spread_success_no_belief_price() {
        let env = Env::default();
        // no belief price, max spread of 100 (0.1 or 10%), offer amount of 10, return amount of 10, spread amount of 1
        // The spread ratio is 10% which is equal to the max spread
        assert_max_spread(&env, None, Decimal::percent(10), 10, 10, 1);
    }

    #[test]
    #[should_panic(expected = "spread exceeds maximum allowed")]
    fn test_assert_max_spread_fail_no_belief_price_max_spread_exceeded() {
        let env = Env::default();
        // no belief price, max spread of 10%, offer amount of 10, return amount of 10, spread amount of 2
        // The spread ratio is 20% which is greater than the max spread
        assert_max_spread(&env, None, Decimal::percent(10), 10, 10, 2);
    }
}
