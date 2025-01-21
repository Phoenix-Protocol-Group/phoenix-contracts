use phoenix::{
    ttl::{INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD},
    utils::{convert_i128_to_u128, convert_u128_to_i128, LiquidityPoolInitInfo},
};
use soroban_sdk::{
    contract, contractimpl, contractmeta, log, panic_with_error, Address, BytesN, Env, String,
};

use crate::{
    error::ContractError,
    math::{calc_y, compute_current_amp, compute_d, scale_value, AMP_PRECISION},
    stake_contract,
    storage::{
        get_amp, get_config, get_greatest_precision, get_precisions, save_amp, save_config,
        save_greatest_precision,
        utils::{self, get_admin_old, is_initialized, set_initialized},
        AmplifierParameters, Asset, Config, PairType, PoolResponse, SimulateReverseSwapResponse,
        SimulateSwapResponse, StableLiquidityPoolInfo, ADMIN,
    },
    token_contract, DECIMAL_PRECISION,
};
use phoenix::{validate_bps, validate_int_parameters};
use soroban_decimal::Decimal;

// Minimum amount of initial LP shares to mint
const MINIMUM_LIQUIDITY_AMOUNT: u128 = 1000;
const MAX_AMP: u64 = 1_000_000;

// Metadata that is added on to the WASM custom section
contractmeta!(
    key = "Description",
    val = "Phoenix Protocol Stable Liquidity Pool"
);

#[contract]
pub struct StableLiquidityPool;

#[allow(dead_code)]
pub trait StableLiquidityPoolTrait {
    // Sets the token contract addresses for this pool
    // token_wasm_hash is the WASM hash of the deployed token contract for the pool share token
    #[allow(clippy::too_many_arguments)]
    fn initialize(
        env: Env,
        stake_wasm_hash: BytesN<32>,
        token_wasm_hash: BytesN<32>,
        lp_init_info: LiquidityPoolInitInfo,
        factory_addr: Address,
        share_token_name: String,
        share_token_symbol: String,
        amp: u64,
        max_allowed_fee_bps: i64,
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
        deadline: Option<u64>,
        min_shares_to_receive: Option<u128>,
    );

    // `offer_asset` is the asset that the user would like to swap for the other token in the pool.
    // `offer_amount` is the amount being sold, with `max_spread_bps` being a safety to make sure you receive at least that amount.
    // swap will transfer the selling token "to" to this contract, and then the contract will transfer the buying token to `sender`.
    // Returns the amount of the token being bought.
    #[allow(clippy::too_many_arguments)]
    fn swap(
        env: Env,
        sender: Address,
        offer_asset: Address,
        offer_amount: i128,
        ask_asset_min_amount: Option<i128>,
        max_spread_bps: Option<i64>,
        deadline: Option<u64>,
        max_allowed_fee_bps: Option<i64>,
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
        deadline: Option<u64>,
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

    fn query_share(env: Env, amount: i128) -> (Asset, Asset);

    fn query_total_issued_lp(env: Env) -> i128;

    fn migrate_admin_key(env: Env) -> Result<(), ContractError>;
}

#[contractimpl]
impl StableLiquidityPoolTrait for StableLiquidityPool {
    #[allow(clippy::too_many_arguments)]
    fn initialize(
        env: Env,
        stake_wasm_hash: BytesN<32>,
        token_wasm_hash: BytesN<32>,
        lp_init_info: LiquidityPoolInitInfo,
        factory_addr: Address,
        share_token_name: String,
        share_token_symbol: String,
        amp: u64,
        max_allowed_fee_bps: i64,
    ) {
        if is_initialized(&env) {
            log!(
                &env,
                "Pool stable: Initialize: initializing contract twice is not allowed"
            );
            panic_with_error!(&env, ContractError::AlreadyInitialized);
        }

        let admin = lp_init_info.admin;
        let swap_fee_bps = lp_init_info.swap_fee_bps;
        let fee_recipient = lp_init_info.fee_recipient;
        let max_allowed_slippage_bps = lp_init_info.max_allowed_slippage_bps;
        let default_slippage_bps = lp_init_info.default_slippage_bps;
        let max_allowed_spread_bps = lp_init_info.max_allowed_spread_bps;
        let token_init_info = lp_init_info.token_init_info;
        let stake_init_info = lp_init_info.stake_init_info;

        validate_bps!(
            swap_fee_bps,
            max_allowed_slippage_bps,
            max_allowed_spread_bps,
            default_slippage_bps,
            max_allowed_fee_bps
        );

        // if the swap_fee_bps is above the threshold, we throw an error
        if swap_fee_bps > max_allowed_fee_bps {
            log!(
                &env,
                "Pool: Initialize: swap fee is higher than the maximum allowed!"
            );
            panic_with_error!(&env, ContractError::SwapFeeBpsOverLimit);
        }

        set_initialized(&env);

        // Token info
        let token_a = token_init_info.token_a;
        let token_b = token_init_info.token_b;
        // Contract info
        let min_bond = stake_init_info.min_bond;
        let min_reward = stake_init_info.min_reward;
        let manager = stake_init_info.manager;

        // Token order validation to make sure only one instance of a pool can exist
        if token_a >= token_b {
            log!(
                &env,
                "Pool Stable: Initialize: First token must be alphabetically smaller than second token"
            );
            panic_with_error!(&env, ContractError::TokenABiggerThanTokenB);
        }

        let decimals = save_greatest_precision(&env, &token_a, &token_b);

        // deploy and initialize token contract
        let share_token_address = utils::deploy_token_contract(
            &env,
            token_wasm_hash,
            &token_a,
            &token_b,
            env.current_contract_address(),
            decimals,
            share_token_name,
            share_token_symbol,
        );

        let stake_contract_address = utils::deploy_stake_contract(&env, stake_wasm_hash);
        stake_contract::Client::new(&env, &stake_contract_address).initialize(
            &admin,
            &share_token_address,
            &min_bond,
            &min_reward,
            &manager,
            &factory_addr,
            &stake_init_info.max_complexity,
        );

        let config = Config {
            token_a: token_a.clone(),
            token_b: token_b.clone(),
            share_token: share_token_address,
            stake_contract: stake_contract_address,
            pool_type: PairType::Stable,
            total_fee_bps: swap_fee_bps,
            fee_recipient,
            max_allowed_slippage_bps,
            default_slippage_bps,
            max_allowed_spread_bps,
        };
        save_config(&env, config);
        let current_time = env.ledger().timestamp();
        if amp == 0 || amp > MAX_AMP {
            log!(&env, "Pool Stable: Initialize: AMP parameter is incorrect");
            panic_with_error!(&env, ContractError::InvalidAMP);
        }

        let amp_precision = amp.checked_mul(AMP_PRECISION).unwrap_or_else(|| {
            log!(&env, "Stable Pool: Initialize: Multiplication overflowed.");
            panic_with_error!(&env, ContractError::ContractMathError);
        });
        save_amp(
            &env,
            AmplifierParameters {
                init_amp: amp_precision,
                init_amp_time: current_time,
                next_amp: amp_precision,
                next_amp_time: current_time,
            },
        );
        utils::save_admin_old(&env, admin);
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
        deadline: Option<u64>,
        min_shares_to_receive: Option<u128>,
    ) {
        if let Some(deadline) = deadline {
            if env.ledger().timestamp() > deadline {
                log!(
                    env,
                    "Pool Stable: Provide Liquidity: Transaction executed after deadline!"
                );
                panic_with_error!(env, ContractError::TransactionAfterTimestampDeadline)
            }
        }

        validate_int_parameters!(desired_a, desired_b);

        // sender needs to authorize the deposit
        sender.require_auth();
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        let config = get_config(&env);
        let greatest_precision = get_greatest_precision(&env);
        let old_balance_a = utils::get_pool_balance_a(&env);
        let old_balance_b = utils::get_pool_balance_b(&env);

        // Check if custom_slippage_bps is more than max_allowed_slippage
        let custom_slippage = custom_slippage_bps.unwrap_or(config.default_slippage_bps);
        if custom_slippage > config.max_allowed_slippage_bps {
            log!(
                    &env,
                    "Pool Stable: ProvideLiquidity: Custom slippage tolerance is more than max allowed slippage tolerance"
                );
            panic_with_error!(env, ContractError::ProvideLiquiditySlippageToleranceTooHigh);
        }

        let amp_parameters = get_amp(&env);
        let amp = compute_current_amp(&env, &amp_parameters);

        let token_a_client = token_contract::Client::new(&env, &config.token_a);
        let token_a_decimals = token_a_client.decimals();
        let token_b_client = token_contract::Client::new(&env, &config.token_b);
        let token_b_decimals = token_b_client.decimals();

        // check the balance before the transfer
        let balance_a_before = token_a_client.balance(&env.current_contract_address());
        let balance_b_before = token_b_client.balance(&env.current_contract_address());

        // transfer tokens from client's wallet to the contract
        token_a_client.transfer(&sender, &env.current_contract_address(), &desired_a);
        token_b_client.transfer(&sender, &env.current_contract_address(), &desired_b);

        // get the balance after transfer
        let balance_a_after = token_a_client.balance(&env.current_contract_address());
        let balance_b_after = token_b_client.balance(&env.current_contract_address());

        // calculate actual amounts received
        let actual_received_a = balance_a_after
            .checked_sub(balance_a_before)
            .unwrap_or_else(|| {
                log!(
                    &env,
                    "Pool Stable: Provide Liquidity: underflow when calculating actual_received_a."
                );
                panic_with_error!(&env, ContractError::ContractMathError);
            });

        let actual_received_b = balance_b_after
            .checked_sub(balance_b_before)
            .unwrap_or_else(|| {
                log!(
                    &env,
                    "Pool Stable: Provide Liquidity: underflow when calculating actual_received_b."
                );
                panic_with_error!(&env, ContractError::ContractMathError);
            });
        // Invariant (D) after deposit added
        let new_balance_a = actual_received_a
            .checked_add(old_balance_a)
            .map(convert_i128_to_u128)
            .unwrap_or_else(|| {
                log!(
                    &env,
                    "Pool Stable: Provide Liquidity: overflow when calculating new_balance_a."
                );
                panic_with_error!(&env, ContractError::ContractMathError);
            });

        let new_balance_b = actual_received_b
            .checked_add(old_balance_b)
            .map(convert_i128_to_u128)
            .unwrap_or_else(|| {
                log!(
                    &env,
                    "Pool Stable: Provide Liquidity: overflow when calculating new_balance_b."
                );
                panic_with_error!(&env, ContractError::ContractMathError);
            });

        let new_invariant = compute_d(
            &env,
            amp as u128,
            &[
                scale_value(&env, new_balance_a, token_a_decimals, DECIMAL_PRECISION),
                scale_value(&env, new_balance_b, token_b_decimals, DECIMAL_PRECISION),
            ],
        );

        let total_shares = utils::get_total_shares(&env);
        let shares = if total_shares == 0 {
            let divisor = 10u128.pow(DECIMAL_PRECISION - greatest_precision);
            let share = new_invariant
                .to_u128()
                .expect("Pool stable: provide_liquidity: conversion to u128 failed")
                .checked_div(divisor)
                .and_then(|quotient| quotient.checked_sub(MINIMUM_LIQUIDITY_AMOUNT))
                .unwrap_or_else(|| {
                    log!(
                        &env,
                        "Pool stable: provide_liquidity: overflow or underflow occurred while calculating share."
                    );
                    panic_with_error!(&env, ContractError::ContractMathError);
                });
            if share == 0 {
                log!(
                    &env,
                    "Pool Stable: ProvideLiquidity: Liquidity amount is too low"
                );
                panic_with_error!(&env, ContractError::LowLiquidity);
            }

            share
        } else {
            let initial_invariant = compute_d(
                &env,
                amp as u128,
                &[
                    scale_value(
                        &env,
                        convert_i128_to_u128(old_balance_a),
                        token_a_decimals,
                        DECIMAL_PRECISION,
                    ),
                    scale_value(
                        &env,
                        convert_i128_to_u128(old_balance_b),
                        token_b_decimals,
                        DECIMAL_PRECISION,
                    ),
                ],
            )
            .to_u128()
            .expect("Pool stable: provide_liquidity: conversion to u128 failed");

            // Calculate the proportion of the change in invariant
            let new_inv = new_invariant
                .to_u128()
                .expect("Pool stable: provide_liquidity: conversion to u128 failed");

            let diff = new_inv.checked_sub(initial_invariant).unwrap_or_else(|| {
                log!(
                    &env,
                    "Pool stable: provide_liquidity: overflow or underflow occurred while calculating invariant_delta."
                );
                panic_with_error!(&env, ContractError::ContractMathError);
            });

            let invariant_delta = convert_u128_to_i128(diff);

            let initial_invariant = convert_u128_to_i128(initial_invariant);
            convert_i128_to_u128(
                total_shares * (Decimal::new(invariant_delta) / Decimal::new(initial_invariant)),
            )
        };

        if let Some(min_shares) = min_shares_to_receive {
            if shares < min_shares {
                log!(
                    env,
                    "Pool Stable: Provide Liquidity: Issued shares are less than the user requsted"
                );
                panic_with_error!(&env, ContractError::IssuedSharesLessThanUserRequested);
            }
        }

        // Now calculate how many new pool shares to mint
        let balance_a = utils::get_balance(&env, &config.token_a);
        let balance_b = utils::get_balance(&env, &config.token_b);

        let shares = convert_u128_to_i128(shares);
        utils::mint_shares(&env, &config.share_token, &sender, shares);
        utils::save_pool_balance_a(&env, balance_a);
        utils::save_pool_balance_b(&env, balance_b);

        env.events()
            .publish(("provide_liquidity", "sender"), sender);
        env.events()
            .publish(("provide_liquidity", "token_a"), &config.token_a);
        env.events()
            .publish(("provide_liquidity", "token_a-amount"), actual_received_a);
        env.events()
            .publish(("provide_liquidity", "token_b"), &config.token_b);
        env.events()
            .publish(("provide_liquidity", "token_b-amount"), actual_received_b);
    }

    #[allow(clippy::too_many_arguments)]
    fn swap(
        env: Env,
        sender: Address,
        offer_asset: Address,
        offer_amount: i128,
        ask_asset_min_amount: Option<i128>,
        max_spread_bps: Option<i64>,
        deadline: Option<u64>,
        max_allowed_fee_bps: Option<i64>,
    ) -> i128 {
        if let Some(deadline) = deadline {
            if env.ledger().timestamp() > deadline {
                log!(
                    env,
                    "Pool Stable: Swap: Transaction executed after deadline!"
                );
                panic_with_error!(env, ContractError::TransactionAfterTimestampDeadline)
            }
        }

        validate_int_parameters!(offer_amount);

        sender.require_auth();
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        do_swap(
            env,
            sender,
            offer_asset,
            offer_amount,
            ask_asset_min_amount,
            max_spread_bps,
            max_allowed_fee_bps,
        )
    }

    fn withdraw_liquidity(
        env: Env,
        sender: Address,
        share_amount: i128,
        min_a: i128,
        min_b: i128,
        deadline: Option<u64>,
    ) -> (i128, i128) {
        if let Some(deadline) = deadline {
            if env.ledger().timestamp() > deadline {
                log!(
                    env,
                    "Pool Stable: Withdraw Liquidity: Transaction executed after deadline!"
                );
                panic_with_error!(env, ContractError::TransactionAfterTimestampDeadline)
            }
        }

        validate_int_parameters!(share_amount, min_a, min_b);

        sender.require_auth();
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        let config = get_config(&env);

        let share_token_client = token_contract::Client::new(&env, &config.share_token);
        share_token_client.transfer(&sender, &env.current_contract_address(), &share_amount);

        let pool_balance_a = utils::get_pool_balance_a(&env);
        let pool_balance_b = utils::get_pool_balance_b(&env);

        let total_shares = utils::get_total_shares(&env);

        if total_shares == 0i128 {
            log!(&env, "Pool Stable: WithdrawLiquidity: Critical error - Total shares are equal to zero before withdrawal!");
            panic_with_error!(env, ContractError::TotalSharesEqualZero);
        }

        let share_ratio = Decimal::from_ratio(share_amount, total_shares);

        let return_amount_a = pool_balance_a * share_ratio;
        let return_amount_b = pool_balance_b * share_ratio;

        if return_amount_a < min_a || return_amount_b < min_b {
            log!(
                &env,
                "Pool Stable: WithdrawLiquidity: Minimum amount of token_a or token_b is not satisfied! min_a: {}, min_b: {}, return_amount_a: {}, return_amount_b: {}",
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

    fn update_config(
        env: Env,
        sender: Address,
        new_admin: Option<Address>,
        total_fee_bps: Option<i64>,
        fee_recipient: Option<Address>,
        max_allowed_slippage_bps: Option<i64>,
        max_allowed_spread_bps: Option<i64>,
    ) {
        if sender != utils::get_admin_old(&env) {
            log!(&env, "Pool Stable: UpdateConfig: Unauthorized");
            panic_with_error!(&env, ContractError::Unauthorized);
        }
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        let mut config = get_config(&env);

        if let Some(new_admin) = new_admin {
            utils::save_admin_old(&env, new_admin);
        }
        if let Some(total_fee_bps) = total_fee_bps {
            validate_bps!(total_fee_bps);
            config.total_fee_bps = total_fee_bps;
        }
        if let Some(fee_recipient) = fee_recipient {
            config.fee_recipient = fee_recipient;
        }
        if let Some(max_allowed_slippage_bps) = max_allowed_slippage_bps {
            validate_bps!(max_allowed_slippage_bps);
            config.max_allowed_slippage_bps = max_allowed_slippage_bps;
        }
        if let Some(max_allowed_spread_bps) = max_allowed_spread_bps {
            validate_bps!(max_allowed_spread_bps);
            config.max_allowed_spread_bps = max_allowed_spread_bps;
        }

        save_config(&env, config);
    }

    fn upgrade(env: Env, new_wasm_hash: BytesN<32>) {
        let admin: Address = utils::get_admin_old(&env);
        admin.require_auth();

        env.deployer().update_current_contract_wasm(new_wasm_hash);
    }

    // Queries

    fn query_config(env: Env) -> Config {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        get_config(&env)
    }

    fn query_share_token_address(env: Env) -> Address {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        get_config(&env).share_token
    }

    fn query_stake_contract_address(env: Env) -> Address {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        get_config(&env).stake_contract
    }

    fn query_pool_info(env: Env) -> PoolResponse {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
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
            stake_address: config.stake_contract,
        }
    }

    fn query_pool_info_for_factory(env: Env) -> StableLiquidityPoolInfo {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
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
            stake_address: config.stake_contract,
        };
        let total_fee_bps = config.total_fee_bps;

        StableLiquidityPoolInfo {
            pool_address: env.current_contract_address(),
            pool_response,
            total_fee_bps,
        }
    }

    fn simulate_swap(env: Env, offer_asset: Address, offer_amount: i128) -> SimulateSwapResponse {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        let config = get_config(&env);

        let pool_balance_a = utils::get_pool_balance_a(&env);
        let pool_balance_b = utils::get_pool_balance_b(&env);

        let (sell_token, pool_balance_sell, buy_token, pool_balance_buy) =
            if offer_asset == config.token_a {
                (
                    config.clone().token_a,
                    pool_balance_a,
                    config.clone().token_b,
                    pool_balance_b,
                )
            } else if offer_asset == config.token_b {
                (
                    config.clone().token_b,
                    pool_balance_b,
                    config.clone().token_a,
                    pool_balance_a,
                )
            } else {
                log!(&env, "Pool Stable: Token offered to swap not found in Pool");
                panic_with_error!(env, ContractError::AssetNotInPool);
            };

        let (ask_amount, spread_amount, commission_amount) = compute_swap(
            &env,
            convert_i128_to_u128(pool_balance_sell),
            get_precisions(&env, &sell_token),
            convert_i128_to_u128(pool_balance_buy),
            get_precisions(&env, &buy_token),
            convert_i128_to_u128(offer_amount),
            config.protocol_fee_rate(),
        );

        let total_return = ask_amount
            .checked_add(commission_amount)
            .and_then(|sum| sum.checked_add(spread_amount))
            .unwrap_or_else(|| {
                log!(&env, "overflow occurred while calculating total_return.");
                panic_with_error!(&env, ContractError::ContractMathError);
            });

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
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        let config = get_config(&env);

        let pool_balance_a = utils::get_pool_balance_a(&env);
        let pool_balance_b = utils::get_pool_balance_b(&env);
        let (sell_token, pool_balance_sell, buy_token, pool_balance_buy) =
            if offer_asset == config.token_a {
                (
                    config.clone().token_a,
                    pool_balance_a,
                    config.clone().token_b,
                    pool_balance_b,
                )
            } else if offer_asset == config.token_b {
                (
                    config.clone().token_b,
                    pool_balance_b,
                    config.clone().token_a,
                    pool_balance_a,
                )
            } else {
                log!(&env, "Pool Stable: Token offered to swap not found in Pool");
                panic_with_error!(env, ContractError::AssetNotInPool);
            };

        let (offer_amount, spread_amount, commission_amount) = compute_offer_amount(
            &env,
            convert_i128_to_u128(pool_balance_sell),
            get_precisions(&env, &sell_token),
            convert_i128_to_u128(pool_balance_buy),
            get_precisions(&env, &buy_token),
            convert_i128_to_u128(ask_amount),
            config.protocol_fee_rate(),
        );

        SimulateReverseSwapResponse {
            offer_amount,
            spread_amount,
            commission_amount,
        }
    }

    fn query_share(env: Env, amount: i128) -> (Asset, Asset) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        let pool_info = Self::query_pool_info(env);
        let total_share = pool_info.asset_lp_share.amount;
        let token_a_amount = pool_info.asset_a.amount;
        let token_b_amount = pool_info.asset_b.amount;

        let mut share_ratio = Decimal::zero();
        if total_share != 0 {
            share_ratio = Decimal::from_ratio(amount, total_share);
        }

        let amount_a = token_a_amount * share_ratio;
        let amount_b = token_b_amount * share_ratio;
        (
            Asset {
                address: pool_info.asset_a.address,
                amount: amount_a,
            },
            Asset {
                address: pool_info.asset_b.address,
                amount: amount_b,
            },
        )
    }

    fn query_total_issued_lp(env: Env) -> i128 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        utils::get_total_shares(&env)
    }

    fn migrate_admin_key(env: Env) -> Result<(), ContractError> {
        let admin = get_admin_old(&env);
        env.storage().instance().set(&ADMIN, &admin);

        Ok(())
    }
}

#[contractimpl]
impl StableLiquidityPool {
    #[allow(dead_code)]
    pub fn update(env: Env, new_wasm_hash: BytesN<32>) {
        let admin = get_admin_old(&env);
        admin.require_auth();

        env.deployer().update_current_contract_wasm(new_wasm_hash);
    }
}

fn do_swap(
    env: Env,
    sender: Address,
    offer_asset: Address,
    offer_amount: i128,
    ask_asset_min_amount: Option<i128>,
    max_spread: Option<i64>,
    max_allowed_fee_bps: Option<i64>,
) -> i128 {
    let config = get_config(&env);

    if let Some(agreed_percentage) = max_allowed_fee_bps {
        if agreed_percentage < config.total_fee_bps {
            log!(
                &env,
                "Pool: do_swap: User agrees to swap at a lower percentage."
            );
            panic_with_error!(&env, ContractError::UserDeclinesPoolFee);
        }
    }

    if offer_asset != config.token_a && offer_asset != config.token_b {
        log!(
            &env,
            "Pool Stable: do swap: Trying to swap wrong asset. Aborting.."
        );
        panic_with_error!(&env, ContractError::IncorrectAssetSwap);
    }

    if let Some(max_spread) = max_spread {
        if !(0..=config.max_allowed_spread_bps).contains(&max_spread) {
            log!(&env, "Pool Stable: do swap: max spread is out of bounds");
            panic_with_error!(&env, ContractError::InvalidBps);
        }
    }

    let max_spread = Decimal::bps(max_spread.map_or_else(|| config.max_allowed_spread_bps, |x| x));

    let pool_balance_a = utils::get_pool_balance_a(&env);
    let pool_balance_b = utils::get_pool_balance_b(&env);

    let (sell_token, pool_balance_sell, buy_token, pool_balance_buy) =
        if offer_asset == config.token_a {
            (
                config.clone().token_a,
                pool_balance_a,
                config.clone().token_b,
                pool_balance_b,
            )
        } else if offer_asset == config.token_b {
            (
                config.clone().token_b,
                pool_balance_b,
                config.clone().token_a,
                pool_balance_a,
            )
        } else {
            log!(&env, "Pool Stable: Token offered to swap not found in Pool");
            panic_with_error!(env, ContractError::AssetNotInPool);
        };

    let (return_amount, spread_amount, commission_amount) = compute_swap(
        &env,
        convert_i128_to_u128(pool_balance_sell),
        get_precisions(&env, &sell_token),
        convert_i128_to_u128(pool_balance_buy),
        get_precisions(&env, &buy_token),
        convert_i128_to_u128(offer_amount),
        config.protocol_fee_rate(),
    );

    if let Some(ask_asset_min_amount) = ask_asset_min_amount {
        if ask_asset_min_amount > return_amount {
            log!(
                &env,
                "Pool Stable: do_swap: Return amount is smaller then expected minimum amount"
            );
            panic_with_error!(&env, ContractError::SwapMinReceivedBiggerThanReturn);
        }
    }

    let return_amount_result = return_amount
        .checked_add(commission_amount)
        .unwrap_or_else(|| {
            log!(&env, "Pool Stable: Do Swap: overflow occured.");
            panic_with_error!(&env, ContractError::ContractMathError);
        });

    assert_max_spread(&env, max_spread, return_amount_result, spread_amount);

    // we check the balance of the transferred token for the contract prior to the transfer
    let balance_before_transfer =
        token_contract::Client::new(&env, &sell_token).balance(&env.current_contract_address());

    // transfer tokens to swap
    token_contract::Client::new(&env, &sell_token).transfer(
        &sender,
        &env.current_contract_address(),
        &offer_amount,
    );

    // get the balance after the transfer
    let balance_after_transfer =
        token_contract::Client::new(&env, &sell_token).balance(&env.current_contract_address());

    // calculate how much did the contract actually got
    //TODO: safe math
    let actual_received_amount = balance_after_transfer - balance_before_transfer;

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
    //TODO: safe math
    let (balance_a, balance_b) = if offer_asset == config.token_a {
        (
            pool_balance_a + actual_received_amount,
            pool_balance_b - commission_amount - return_amount,
        )
    } else {
        (
            pool_balance_a - commission_amount - return_amount,
            pool_balance_b + actual_received_amount,
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
/// * `max_spread` - The maximum allowed spread (slippage) as a fraction of the return amount.
/// * `return_amount` - The amount of tokens that the user receives in return.
/// * `spread_amount` - The spread (slippage) amount, i.e., the difference between the expected and actual return.
/// # Returns
/// * An error if the spread exceeds the maximum allowed, otherwise Ok.
pub fn assert_max_spread(env: &Env, max_spread: Decimal, return_amount: i128, spread_amount: i128) {
    // Calculate the spread ratio, the fraction of the return that is due to spread
    let spread_ratio = Decimal::from_ratio(spread_amount, return_amount);

    if spread_ratio > max_spread {
        log!(env, "Pool Stable: Spread exceeds maximum allowed");
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
///
/// Returns a tuple containing the following values:
/// - The resulting amount of ask assets after the swap minus the commission amount.
/// - The spread amount, representing the difference between the expected and actual swap amounts.
/// - The commission amount, representing the fees charged for the swap.
pub fn compute_swap(
    env: &Env,
    offer_pool: u128,
    offer_pool_precision: u32,
    ask_pool: u128,
    ask_pool_precision: u32,
    offer_amount: u128,
    commission_rate: Decimal,
) -> (i128, i128, i128) {
    let amp_parameters = get_amp(env);
    let amp = compute_current_amp(env, &amp_parameters);

    let greatest_precision = get_greatest_precision(env);

    let new_ask_pool = calc_y(
        env,
        amp as u128,
        scale_value(
            //TODO: safe math
            env,
            offer_pool + offer_amount,
            greatest_precision,
            DECIMAL_PRECISION,
        ),
        &[
            scale_value(env, offer_pool, offer_pool_precision, DECIMAL_PRECISION),
            scale_value(env, ask_pool, ask_pool_precision, DECIMAL_PRECISION),
        ],
        greatest_precision,
    );

    //TODO: safe math
    let return_amount = ask_pool - new_ask_pool;
    // We consider swap rate 1:1 in stable swap thus any difference is considered as spread.
    let spread_amount = if offer_amount > return_amount {
        //TODO: safe math
        convert_u128_to_i128(offer_amount - return_amount)
    } else {
        // saturating sub equivalent
        0
    };
    let return_amount = convert_u128_to_i128(return_amount);
    let commission_amount = return_amount * commission_rate;
    // Because of issue #211
    //TODO: safe math
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
    offer_pool: u128,
    offer_pool_precision: u32,
    ask_pool: u128,
    ask_pool_precision: u32,
    ask_amount: u128,
    commission_rate: Decimal,
) -> (i128, i128, i128) {
    let amp_parameters = get_amp(env);
    let amp = compute_current_amp(env, &amp_parameters);

    let one_minus_commission = Decimal::one() - commission_rate;
    let inv_one_minus_commission = Decimal::one() / one_minus_commission;
    let before_commission = inv_one_minus_commission * convert_u128_to_i128(ask_amount);

    let greatest_precision = get_greatest_precision(env);

    let new_offer_pool = calc_y(
        env,
        amp as u128,
        scale_value(
            //TODO: safe math
            env,
            ask_pool - convert_i128_to_u128(before_commission),
            greatest_precision,
            DECIMAL_PRECISION,
        ),
        &[
            scale_value(env, offer_pool, offer_pool_precision, DECIMAL_PRECISION),
            scale_value(env, ask_pool, ask_pool_precision, DECIMAL_PRECISION),
        ],
        greatest_precision,
    );

    //TODO: safe math
    let offer_amount = new_offer_pool - offer_pool;

    //TODO: safe math
    let ask_before_commission = convert_u128_to_i128(ask_amount) * inv_one_minus_commission;
    // We consider swap rate 1:1 in stable swap thus any difference is considered as spread.
    let spread_amount = if offer_amount > ask_amount {
        //TODO: safe math
        offer_amount - ask_amount
    } else {
        // saturating sub equivalent
        0
    };

    // Calculate the commission amount
    let commission_amount: i128 = ask_before_commission * commission_rate;

    (
        convert_u128_to_i128(offer_amount),
        convert_u128_to_i128(spread_amount),
        commission_amount,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assert_max_spread_success() {
        let env = Env::default();
        // Test case that should pass:
        // max spread of 10%, offer amount of 100k, return amount of 100k and 1 unit, spread amount of 1
        // The spread ratio is 10% which is equal to the max spread
        assert_max_spread(&env, Decimal::percent(10), 100_001, 1);
    }

    #[test]
    #[should_panic(expected = "Spread exceeds maximum allowed")]
    fn test_assert_max_spread_fail_max_spread_exceeded() {
        let env = Env::default();

        let max_spread = Decimal::percent(10); // 10% is the maximum allowed spread
        let return_amount = 100; // These values are chosen such that the spread ratio will be more than 10%
        let spread_amount = 35;

        assert_max_spread(&env, max_spread, return_amount, spread_amount);
    }

    #[test]
    fn test_assert_max_spread_success_no_belief_price() {
        let env = Env::default();
        // max spread of 100 (0.1 or 10%), return amount of 10, spread amount of 1
        // The spread ratio is 10% which is equal to the max spread
        assert_max_spread(&env, Decimal::percent(10), 10, 1);
    }

    #[test]
    #[should_panic(expected = "Spread exceeds maximum allowed")]
    fn test_assert_max_spread_fail_no_belief_price_max_spread_exceeded() {
        let env = Env::default();
        // max spread of 10%, return amount of 10, spread amount of 2
        // The spread ratio is 20% which is greater than the max spread
        assert_max_spread(&env, Decimal::percent(10), 10, 2);
    }
}
