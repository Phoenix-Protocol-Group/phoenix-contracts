use soroban_sdk::{
    contract, contractimpl, contractmeta, log, panic_with_error, Address, BytesN, Env, IntoVal,
};

use num_integer::Roots;

use crate::{
    error::ContractError,
    stake_contract,
    storage::{
        get_config, save_config, utils,
        utils::{is_initialized, set_initialized},
        validate_fee_bps, Asset, ComputeSwap, Config, LiquidityPoolInfo, PairType, PoolResponse,
        SimulateReverseSwapResponse, SimulateSwapResponse,
    },
    token_contract,
};
use decimal::Decimal;
use phoenix::{
    utils::{is_approx_ratio, LiquidityPoolInitInfo},
    validate_bps, validate_int_parameters,
};

// Metadata that is added on to the WASM custom section
contractmeta!(
    key = "Description",
    val = "Phoenix Protocol XYK Liquidity Pool"
);

#[contract]
pub struct LiquidityPool;

pub trait LiquidityPoolTrait {
    // Sets the token contract addresses for this pool
    // token_wasm_hash is the WASM hash of the deployed token contract for the pool share token
    #[allow(clippy::too_many_arguments)]
    fn initialize(
        env: Env,
        stake_wasm_hash: BytesN<32>,
        token_wasm_hash: BytesN<32>,
        lp_init_info: LiquidityPoolInitInfo,
    );

    // Deposits token_a and token_b. Also mints pool shares for the "to" Identifier. The amount minted
    // is determined based on the difference between the reserves stored by this contract, and
    // the actual balance of token_a and token_b for this contract.
    fn provide_liquidity(
        env: Env,
        depositor: Address,
        desired_a: Option<i128>,
        min_a: Option<i128>,
        desired_b: Option<i128>,
        min_b: Option<i128>,
        custom_slippage_bps: Option<i64>,
    );

    // `offer_asset` is the asset that the user would like to swap for the other token in the pool.
    // `offer_amount` is the amount being sold, with `max_spread_bps` being a safety to make sure you receive at least that amount.
    // swap will transfer the selling token "to" to this contract, and then the contract will transfer the buying token to `sender`.
    // Returns the amount of the token being bought.
    fn swap(
        env: Env,
        sender: Address,
        // FIXM: Disable Referral struct
        // referral: Option<Referral>,
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
    #[allow(clippy::too_many_arguments)]
    fn update_config(
        env: Env,
        new_admin: Option<Address>,
        total_fee_bps: Option<i64>,
        fee_recipient: Option<Address>,
        max_allowed_slippage_bps: Option<i64>,
        max_allowed_spread_bps: Option<i64>,
        max_referral_bps: Option<i64>,
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

    fn query_pool_info_for_factory(env: Env) -> LiquidityPoolInfo;

    // Simulate swap transaction
    fn simulate_swap(env: Env, offer_asset: Address, sell_amount: i128) -> SimulateSwapResponse;

    // Simulate reverse swap transaction
    fn simulate_reverse_swap(
        env: Env,
        ask_asset: Address,
        ask_amount: i128,
    ) -> SimulateReverseSwapResponse;
}

#[contractimpl]
impl LiquidityPoolTrait for LiquidityPool {
    #[allow(clippy::too_many_arguments)]
    fn initialize(
        env: Env,
        stake_wasm_hash: BytesN<32>,
        token_wasm_hash: BytesN<32>,
        lp_init_info: LiquidityPoolInitInfo,
    ) {
        if is_initialized(&env) {
            panic!("Liquidity Pool: Initialize: initializing contract twice is not allowed");
        }

        let admin = lp_init_info.admin;
        let share_token_decimals = lp_init_info.share_token_decimals;
        let swap_fee_bps = lp_init_info.swap_fee_bps;
        let fee_recipient = lp_init_info.fee_recipient;
        let max_allowed_slippage_bps = lp_init_info.max_allowed_slippage_bps;
        let max_allowed_spread_bps = lp_init_info.max_allowed_spread_bps;
        let max_referral_bps = lp_init_info.max_referral_bps;
        let token_init_info = lp_init_info.token_init_info;
        let stake_init_info = lp_init_info.stake_init_info;

        validate_bps!(
            swap_fee_bps,
            max_allowed_slippage_bps,
            max_allowed_spread_bps,
            max_referral_bps
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
            max_referral_bps,
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
    }

    fn provide_liquidity(
        env: Env,
        sender: Address,
        desired_a: Option<i128>,
        min_a: Option<i128>,
        desired_b: Option<i128>,
        min_b: Option<i128>,
        custom_slippage_bps: Option<i64>,
    ) {
        validate_int_parameters!(desired_a, min_a, desired_b, min_b);

        // sender needs to authorize the deposit
        sender.require_auth();

        let config = get_config(&env);
        let pool_balance_a = utils::get_pool_balance_a(&env);
        let pool_balance_b = utils::get_pool_balance_b(&env);

        // Check if custom_slippage_bps is more than max_allowed_slippage
        if let Some(custom_slippage) = custom_slippage_bps {
            if custom_slippage > config.max_allowed_slippage_bps {
                log!(
                    &env,
                    "Pool: ProvideLiquidity: Custom slippage tolerance is more than max allowed slippage toleranc"
                );
                panic_with_error!(env, ContractError::ProvideLiquiditySlippageToleranceTooHigh);
            }
        }

        // Check if both tokens are provided, one token is provided, or none are provided
        let amounts = match (desired_a, desired_b) {
            // Both tokens are provided
            (Some(a), Some(b)) if a > 0 && b > 0 => {
                // Calculate deposit amounts
                utils::get_deposit_amounts(
                    &env,
                    a,
                    min_a,
                    b,
                    min_b,
                    pool_balance_a,
                    pool_balance_b,
                    Decimal::bps(custom_slippage_bps.unwrap_or(100)),
                )
            }
            // Only token A is provided
            (Some(a), None) if a > 0 => {
                let (a_for_swap, b_from_swap) = split_deposit_based_on_pool_ratio(
                    &env,
                    &config,
                    pool_balance_a,
                    pool_balance_b,
                    a,
                    &config.token_a,
                );
                do_swap(
                    env.clone(),
                    sender.clone(),
                    // FIXM: Disable Referral struct
                    // None,
                    config.clone().token_a,
                    a_for_swap,
                    None,
                    None,
                );
                // return: rest of Token A amount, simulated result of swap of portion A
                (a - a_for_swap, b_from_swap)
            }
            // Only token B is provided
            (None, Some(b)) if b > 0 => {
                let (b_for_swap, a_from_swap) = split_deposit_based_on_pool_ratio(
                    &env,
                    &config,
                    pool_balance_a,
                    pool_balance_b,
                    b,
                    &config.token_b,
                );
                do_swap(
                    env.clone(),
                    sender.clone(),
                    // FIXM: Disable Referral struct
                    // None,
                    config.clone().token_b,
                    b_for_swap,
                    None,
                    None,
                );
                // return: simulated result of swap of portion B, rest of Token B amount
                (a_from_swap, b - b_for_swap)
            }
            // None or invalid amounts are provided
            _ => {
                log!(
                    &env,
                        "Pool: ProvideLiquidity: At least one token must be provided and must be bigger then 0!"
                );
                panic_with_error!(
                    env,
                    ContractError::ProvideLiquidityAtLeastOneTokenMustBeBiggerThenZero
                );
            }
        };

        let token_a_client = token_contract::Client::new(&env, &config.token_a);
        let token_b_client = token_contract::Client::new(&env, &config.token_b);

        // Move tokens from client's wallet to the contract
        token_a_client.transfer(&sender, &env.current_contract_address(), &(amounts.0));
        token_b_client.transfer(&sender, &env.current_contract_address(), &(amounts.1));

        let pool_balance_a = utils::get_pool_balance_a(&env);
        let pool_balance_b = utils::get_pool_balance_b(&env);

        // Now calculate how many new pool shares to mint
        let balance_a = utils::get_balance(&env, &config.token_a);
        let balance_b = utils::get_balance(&env, &config.token_b);
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
            &config.share_token,
            &sender,
            new_total_shares - total_shares,
        );
        utils::save_pool_balance_a(&env, balance_a);
        utils::save_pool_balance_b(&env, balance_b);

        env.events()
            .publish(("provide_liquidity", "sender"), sender);
        env.events()
            .publish(("provide_liquidity", "token_a"), &config.token_a);
        env.events()
            .publish(("provide_liquidity", "token_a-amount"), amounts.0);
        env.events()
            .publish(("provide_liquidity", "token_b"), &config.token_b);
        env.events()
            .publish(("provide_liquidity", "token_b-amount"), amounts.1);
    }

    fn swap(
        env: Env,
        sender: Address,
        // FIXM: Disable Referral struct
        // referral: Option<Referral>,
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
            // referral,
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
                "Pool: WithdrawLiquidity: Minimum amount of token_a or token_b is not satisfied! min_a: {}, min_b: {}, return_amount_a: {}, return_amount_b: {}",
                min_a,
                min_b,
                return_amount_a,
                return_amount_b
            );
            panic_with_error!(
                env,
                ContractError::WithdrawLiquidityMinimumAmountOfAOrBIsNotSatisfied
            );
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

    #[allow(clippy::too_many_arguments)]
    fn update_config(
        env: Env,
        new_admin: Option<Address>,
        total_fee_bps: Option<i64>,
        fee_recipient: Option<Address>,
        max_allowed_slippage_bps: Option<i64>,
        max_allowed_spread_bps: Option<i64>,
        max_referral_bps: Option<i64>,
    ) {
        let admin: Address = utils::get_admin(&env);
        admin.require_auth();

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
        if let Some(max_referral_bps) = max_referral_bps {
            config.max_referral_bps = max_referral_bps;
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

    fn query_pool_info_for_factory(env: Env) -> LiquidityPoolInfo {
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

        LiquidityPoolInfo {
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

        let compute_swap: ComputeSwap = compute_swap(
            pool_balance_offer,
            pool_balance_ask,
            offer_amount,
            config.protocol_fee_rate(),
            0i64,
        );

        let total_return = compute_swap.return_amount
            + compute_swap.commission_amount
            + compute_swap.spread_amount;

        SimulateSwapResponse {
            ask_amount: compute_swap.return_amount,
            commission_amount: compute_swap.commission_amount,
            spread_amount: compute_swap.spread_amount,
            total_return,
        }
    }

    fn simulate_reverse_swap(
        env: Env,
        ask_asset: Address,
        ask_amount: i128,
    ) -> SimulateReverseSwapResponse {
        let config = get_config(&env);

        let pool_balance_a = utils::get_pool_balance_a(&env);
        let pool_balance_b = utils::get_pool_balance_b(&env);
        let (pool_balance_offer, pool_balance_ask) = if ask_asset == config.token_b {
            (pool_balance_a, pool_balance_b)
        } else {
            (pool_balance_b, pool_balance_a)
        };

        let (offer_amount, spread_amount, commission_amount) = compute_offer_amount(
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
    // FIXM: Disable Referral struct
    // referral: Option<Referral>,
    offer_asset: Address,
    offer_amount: i128,
    belief_price: Option<i64>,
    max_spread: Option<i64>,
) -> i128 {
    let config = get_config(&env);
    // FIXM: Disable Referral struct
    // if let Some(referral) = &referral {
    //     if referral.fee > config.max_referral_bps {
    //         panic!("Pool: Swap: Trying to swap with more than the allowed referral fee");
    //     }
    // }

    let belief_price = belief_price.map(Decimal::percent);
    let max_spread = Decimal::bps(max_spread.map_or_else(|| config.max_allowed_spread_bps, |x| x));

    let pool_balance_a = utils::get_pool_balance_a(&env);
    let pool_balance_b = utils::get_pool_balance_b(&env);

    let (pool_balance_sell, pool_balance_buy) = if offer_asset == config.token_a {
        (pool_balance_a, pool_balance_b)
    } else {
        (pool_balance_b, pool_balance_a)
    };

    // FIXM: Disable Referral struct
    // let referral_fee_bps = match referral {
    //     Some(ref referral) => referral.clone().fee,
    //     None => 0,
    // };
    let referral_fee_bps = 0;

    // 1. We calculate the referral_fee below. If none referral fee will be 0
    let compute_swap: ComputeSwap = compute_swap(
        pool_balance_sell,
        pool_balance_buy,
        offer_amount,
        config.protocol_fee_rate(),
        referral_fee_bps,
    );

    let total_return_amount = compute_swap.return_amount
        + compute_swap.commission_amount
        + compute_swap.referral_fee_amount;

    assert_max_spread(
        &env,
        belief_price,
        max_spread,
        offer_amount,
        total_return_amount,
        compute_swap.spread_amount,
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
        &compute_swap.return_amount,
    );

    // send commission to fee recipient
    token_contract::Client::new(&env, &buy_token).transfer(
        &env.current_contract_address(),
        &config.fee_recipient,
        &compute_swap.commission_amount,
    );

    // 2. If referral is present and return amount is larger than 0 we send referral fee commision
    //    to fee recipient
    // FIXM: Disable Referral struct
    // if let Some(Referral { address, fee }) = referral {
    //     if fee > 0 {
    //         token_contract::Client::new(&env, &buy_token).transfer(
    //             &env.current_contract_address(),
    //             &address,
    //             &compute_swap.referral_fee_amount,
    //         );
    //     }
    // }

    // user is offering to sell A, so they will receive B
    // A balance is bigger, B balance is smaller
    let (balance_a, balance_b) = if offer_asset == config.token_a {
        (
            pool_balance_a + offer_amount,
            pool_balance_b
                - compute_swap.commission_amount
                - compute_swap.referral_fee_amount
                - compute_swap.return_amount,
        )
    } else {
        (
            pool_balance_a
                - compute_swap.commission_amount
                - compute_swap.referral_fee_amount
                - compute_swap.return_amount,
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
        .publish(("swap", "return_amount"), compute_swap.return_amount);
    env.events()
        .publish(("swap", "spread_amount"), compute_swap.spread_amount);
    env.events().publish(
        ("swap", "referral_fee_amount"),
        compute_swap.referral_fee_amount,
    );
    compute_swap.return_amount
}

/// This function divides the deposit in such a way that when swapping it for the other token,
/// the resulting amounts of tokens maintain the current pool's ratio.
/// * `a_pool` - The current amount of Token A in the liquidity pool.
/// * `b_pool` - The current amount of Token B in the liquidity pool.
/// * `deposit` - The total amount of tokens that the user wants to deposit into the liquidity pool.
/// * `sell_a` - A boolean that indicates whether the deposit is in Token A (if true) or in Token B (if false).
/// # Returns
/// * A tuple `(final_offer_amount, final_ask_amount)`, where `final_offer_amount` is the amount of deposit tokens
///   to be swapped, and `final_ask_amount` is the amount of the other tokens that will be received in return.
fn split_deposit_based_on_pool_ratio(
    env: &Env,
    config: &Config,
    a_pool: i128,
    b_pool: i128,
    deposit: i128,
    offer_asset: &Address,
) -> (i128, i128) {
    // Validate the inputs
    if a_pool <= 0 || b_pool <= 0 || deposit <= 0 {
        log!(
            env,
            "Pool: split_deposit_based_on_pool_ratio: Both pools and deposit must be a positive!"
        );
        panic_with_error!(
            env,
            ContractError::SplitDepositBothPoolsAndDepositMustBePositive
        );
    }

    // Calculate the current ratio in the pool
    let target_ratio = Decimal::from_ratio(b_pool, a_pool);
    // Define boundaries for binary search algorithm
    let mut low = 0;
    let mut high = deposit;

    // Tolerance is the smallest difference in deposit that we care about
    let tolerance = 500;

    let mut final_offer_amount = deposit; // amount of deposit tokens to be swapped
    let mut final_ask_amount = 0; // amount of other tokens to be received

    while high - low > tolerance {
        let mid = (low + high) / 2; // Calculate middle point

        // Simulate swap to get amount of other tokens to be received for `mid` amount of deposit tokens
        let SimulateSwapResponse {
            ask_amount,
            spread_amount: _,
            commission_amount: _,
            total_return: _,
        } = LiquidityPool::simulate_swap(env.clone(), offer_asset.clone(), mid);

        // Update final amounts
        final_offer_amount = mid;
        final_ask_amount = ask_amount;

        // Calculate the ratio that would result from swapping `mid` deposit tokens
        let ratio = if offer_asset == &config.token_a {
            Decimal::from_ratio(ask_amount, deposit - mid)
        } else {
            Decimal::from_ratio(deposit - mid, ask_amount)
        };

        // If the resulting ratio is approximately equal (1%) to the target ratio, break the loop
        if is_approx_ratio(ratio, target_ratio, Decimal::percent(1)) {
            break;
        }
        // Update boundaries for the next iteration of the binary search
        if ratio > target_ratio {
            if offer_asset == &config.token_a {
                high = mid;
            } else {
                low = mid;
            }
        } else if offer_asset == &config.token_a {
            low = mid;
        } else {
            high = mid;
        };
    }
    (final_offer_amount, final_ask_amount)
}

/// This function asserts that the slippage does not exceed the provided tolerance.
/// # Arguments
/// * `slippage_tolerance` - An optional user-provided slippage tolerance as basis points.
/// * `deposits` - The amounts of tokens that the user deposits into each of the two pools.
/// * `pools` - The amounts of tokens in each of the two pools before the deposit.
/// * `max_allowed_slippage` - The maximum allowed slippage as a decimal.
/// # Returns
/// * An error if the slippage exceeds the tolerance or if the tolerance itself exceeds the maximum allowed,
///   otherwise Ok.
#[allow(dead_code)]
fn assert_slippage_tolerance(
    env: &Env,
    slippage_tolerance: Option<i64>,
    deposits: &[i128; 2],
    pools: &[i128; 2],
    max_allowed_slippage: Decimal,
) {
    let default_slippage = Decimal::percent(1); // Representing 1% as the default slippage tolerance

    // If user provided a slippage tolerance, convert it from basis points to a decimal
    // Otherwise, use the default slippage tolerance
    let slippage_tolerance = if let Some(slippage_tolerance) = slippage_tolerance {
        Decimal::bps(slippage_tolerance)
    } else {
        default_slippage
    };
    if slippage_tolerance > max_allowed_slippage {
        log!(env, "Slippage tolerance exceeds the maximum allowed value");
        panic!(
            "Pool: Assert slippage tolerance: slippage tolerance exceeds the maximum allowed value"
        );
    }

    // Calculate the limit below which the deposit-to-pool ratio must not fall for each token
    let one_minus_slippage_tolerance = Decimal::one() - slippage_tolerance;
    let deposits: [i128; 2] = [deposits[0], deposits[1]];
    let pools: [i128; 2] = [pools[0], pools[1]];

    // Ensure each price does not change more than what the slippage tolerance allows
    if deposits[0] * pools[1] * one_minus_slippage_tolerance
        > deposits[1] * pools[0] * Decimal::one()
        || deposits[1] * pools[0] * one_minus_slippage_tolerance
            > deposits[0] * pools[1] * Decimal::one()
    {
        log!(
            env,
            "Slippage tolerance violated. Deposits: 0: {} 1: {}, Pools: 0: {} 1: {}",
            deposits[0],
            deposits[1],
            pools[0],
            pools[1]
        );
        panic!("Pool: Assert slippage tolerance: slippage tolerance violated");
    }
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
        panic_with_error!(env, ContractError::SpreadExceedsLimit);
    }
}

/// Computes the result of a swap operation.
///
/// Arguments:
/// - `offer_pool`: Total amount of offer assets in the pool.
/// - `ask_pool`: Total amount of ask assets in the pool.
/// - `offer_amount`: Amount of offer assets to swap.
/// - `commission_rate`: Total amount of fees charged for the swap.
/// - `referral_fee`: Amount of fee for the referral
///
/// Returns a tuple containing the following values:
/// - The resulting amount of ask assets after the swap.
/// - The spread amount, representing the difference between the expected and actual swap amounts.
/// - The commission amount, representing the fees charged for the swap.
/// - The referral comission fee.
pub fn compute_swap(
    offer_pool: i128,
    ask_pool: i128,
    offer_amount: i128,
    commission_rate: Decimal,
    referral_fee: i64,
) -> ComputeSwap {
    // Calculate the cross product of offer_pool and ask_pool
    let cp: i128 = offer_pool * ask_pool;

    // Calculate the resulting amount of ask assets after the swap
    // Return amount calculation based on the AMM model's invariant,
    // which ensures the product of the amounts of the two assets remains constant before and after a trade.
    let return_amount: i128 = ask_pool - (cp / (offer_pool + offer_amount));
    // Calculate the spread amount, representing the difference between the expected and actual swap amounts
    let spread_amount: i128 = (offer_amount * ask_pool / offer_pool) - return_amount;

    let commission_amount: i128 = return_amount * commission_rate;

    // Deduct the commission (minus the part that goes to the protocol) from the return amount
    let return_amount: i128 = return_amount - commission_amount;
    let referral_fee_amount: i128 = return_amount * Decimal::bps(referral_fee);

    let return_amount: i128 = return_amount - referral_fee_amount;

    ComputeSwap {
        return_amount,
        spread_amount,
        commission_amount,
        referral_fee_amount,
    }
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
) -> (i128, i128, i128) {
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

    (offer_amount, spread_amount, commission_amount)
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
    }

    #[test]
    #[should_panic(expected = "slippage tolerance exceeds the maximum allowed value")]
    fn test_assert_slippage_tolerance_fail_tolerance_too_high() {
        let env = Env::default();
        // Test case that should fail due to slippage tolerance being too high
        let max_allowed_slippage = Decimal::bps(5_000i64);
        assert_slippage_tolerance(
            &env,
            Some(60_000),
            &[10, 20],
            &[30, 60],
            max_allowed_slippage,
        );
    }

    #[test]
    #[should_panic(expected = "slippage tolerance violated")]
    fn test_assert_slippage_tolerance_fail_slippage_violated() {
        let env = Env::default();
        let max_allowed_slippage = Decimal::bps(5_000i64);
        // The price changes from 10/15 (0.67) to 40/40 (1.00), violating the 10% slippage tolerance
        assert_slippage_tolerance(
            &env,
            Some(1_000),
            &[10, 15],
            &[40, 40],
            max_allowed_slippage,
        );
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
        );
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #1)")]
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
    #[should_panic(expected = "HostError: Error(Contract, #1)")]
    fn test_assert_max_spread_fail_no_belief_price_max_spread_exceeded() {
        let env = Env::default();
        // no belief price, max spread of 10%, offer amount of 10, return amount of 10, spread amount of 2
        // The spread ratio is 20% which is greater than the max spread
        assert_max_spread(&env, None, Decimal::percent(10), 10, 10, 2);
    }

    #[test]
    fn test_compute_swap_pass() {
        let result = compute_swap(1000, 2000, 100, Decimal::percent(10), 0i64); // 10% commission rate
        let expected_compute_swap = ComputeSwap {
            return_amount: 164,
            spread_amount: 18,
            commission_amount: 18,
            referral_fee_amount: 0,
        };

        assert_eq!(result, expected_compute_swap); // Expected return amount, spread, commission and referral fee commission
    }

    #[test]
    fn test_compute_swap_pass_with_referral_fee() {
        // 10% commission rate + 15% referral fee
        // return_amount would be 164, but after we deduct 15% out of it we get to 139.4 rounded to
        // the closest number 140
        let result = compute_swap(1000, 2000, 100, Decimal::percent(10), 1_500i64);
        let expected_compute_swap = ComputeSwap {
            return_amount: 140,
            spread_amount: 18,
            commission_amount: 18,
            referral_fee_amount: 24,
        };

        assert_eq!(result, expected_compute_swap); // Expected return amount, spread, commission and referral fee commission
    }

    #[test]
    fn test_compute_swap_full_commission() {
        let result = compute_swap(1000, 2000, 100, Decimal::one(), 0i64); // 100% commission rate should lead to return_amount being 0
        let expected_compute_swap = ComputeSwap {
            return_amount: 0,
            spread_amount: 18,
            commission_amount: 182,
            referral_fee_amount: 0,
        };

        assert_eq!(result, expected_compute_swap);
    }

    #[test]
    fn test_compute_offer_amount() {
        let offer_pool = 1000000;
        let ask_pool = 1000000;
        let commission_rate = Decimal::percent(10);
        let ask_amount = 1000;

        let result = compute_offer_amount(offer_pool, ask_pool, ask_amount, commission_rate);

        // Test that the offer amount is less than the original pool size, due to commission
        assert!(result.0 < offer_pool);

        // Test that the spread amount is non-negative
        assert!(result.1 >= 0);

        // Test that the commission amount is exactly 10% of the offer amount
        assert_eq!(result.2, result.0 * Decimal::percent(10));
    }
}
