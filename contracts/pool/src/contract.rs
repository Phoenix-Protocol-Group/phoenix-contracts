use soroban_sdk::{
    contract, contractimpl, contractmeta, log, panic_with_error, Address, BytesN, Env, String, U256,
};

use num_integer::Roots;

use crate::{
    error::ContractError,
    stake_contract,
    storage::{
        get_config, get_default_slippage_bps, save_config, save_default_slippage_bps,
        utils::{self, get_admin_old, is_initialized, set_initialized},
        Asset, ComputeSwap, Config, DataKey, LiquidityPoolInfo, PairType, PoolResponse,
        SimulateReverseSwapResponse, SimulateSwapResponse, ADMIN, CONFIG, DEFAULT_SLIPPAGE_BPS,
        XYK_POOL_KEY,
    },
    token_contract,
};
use phoenix::{
    ttl::{
        INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL, PERSISTENT_RENEWAL_THRESHOLD,
        PERSISTENT_TARGET_TTL,
    },
    utils::{convert_i128_to_u128, is_approx_ratio, LiquidityPoolInitInfo},
    validate_bps, validate_int_parameters,
};
use soroban_decimal::Decimal;

/// Minimum initial LP share
const MINIMUM_LIQUIDITY_AMOUNT: i128 = 1_000i128;

// Metadata that is added on to the WASM custom section
contractmeta!(
    key = "Description",
    val = "Phoenix Protocol XYK Liquidity Pool"
);

#[contract]
pub struct LiquidityPool;

#[allow(dead_code)]
pub trait LiquidityPoolTrait {
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
        default_slippage_bps: i64,
        max_allowed_fee_bps: i64,
    );

    // Deposits token_a and token_b. Also mints pool shares for the "to" Identifier. The amount minted
    // is determined based on the difference between the reserves stored by this contract, and
    // the actual balance of token_a and token_b for this contract.
    #[allow(clippy::too_many_arguments)]
    fn provide_liquidity(
        env: Env,
        depositor: Address,
        desired_a: Option<i128>,
        min_a: Option<i128>,
        desired_b: Option<i128>,
        min_b: Option<i128>,
        custom_slippage_bps: Option<i64>,
        deadline: Option<u64>,
        auto_stake: bool,
    );

    // `offer_asset` is the asset that the user would like to swap for the other token in the pool.
    // `offer_amount` is the amount being sold, with `max_spread_bps` being a safety to make sure you receive at least that amount.
    // swap will transfer the selling token "to" to this contract, and then the contract will transfer the buying token to `sender`.
    // Returns the amount of the token being bought.
    #[allow(clippy::too_many_arguments)]
    fn swap(
        env: Env,
        sender: Address,
        // FIXM: Disable Referral struct
        // referral: Option<Referral>,
        offer_asset: Address,
        offer_amount: i128,
        // Minimum amount of the ask token user expects to receive
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
    fn upgrade(e: Env, new_wasm_hash: BytesN<32>, new_default_slippage_bps: i64);

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

    fn query_share(env: Env, amount: i128) -> (Asset, Asset);

    fn query_total_issued_lp(env: Env) -> i128;

    fn migrate_admin_key(env: Env) -> Result<(), ContractError>;

    fn propose_admin(
        env: Env,
        new_admin: Address,
        time_limit: Option<u64>,
    ) -> Result<Address, ContractError>;

    fn revoke_admin_change(env: Env) -> Result<(), ContractError>;

    fn accept_admin(env: Env) -> Result<Address, ContractError>;
}

#[contractimpl]
impl LiquidityPoolTrait for LiquidityPool {
    #[allow(clippy::too_many_arguments)]
    fn initialize(
        env: Env,
        stake_wasm_hash: BytesN<32>,
        token_wasm_hash: BytesN<32>,
        lp_init_info: LiquidityPoolInitInfo,
        factory_addr: Address,
        share_token_name: String,
        share_token_symbol: String,
        default_slippage_bps: i64,
        max_allowed_fee_bps: i64,
    ) {
        if is_initialized(&env) {
            log!(
                &env,
                "Pool: Initialize: initializing contract twice is not allowed"
            );
            panic_with_error!(&env, ContractError::AlreadyInitialized);
        }

        let admin = lp_init_info.admin;
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
            max_referral_bps,
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
        // Stake info
        let min_bond = stake_init_info.min_bond;
        let min_reward = stake_init_info.min_reward;
        let manager = stake_init_info.manager;

        // Token order validation to make sure only one instance of a pool can exist
        if token_a >= token_b {
            log!(
                &env,
                "Pool: Initialize: First token must be alphabetically smaller than second token"
            );
            panic_with_error!(&env, ContractError::TokenABiggerThanTokenB);
        }

        let precision1 = token_contract::Client::new(&env, &token_a).decimals();
        let precision2 = token_contract::Client::new(&env, &token_b).decimals();
        let max_precision_decimals: u32 = if precision1 > precision2 {
            precision1
        } else {
            precision2
        };

        // deploy and initialize token contract
        let share_token_address = utils::deploy_token_contract(
            &env,
            token_wasm_hash.clone(),
            &token_a,
            &token_b,
            env.current_contract_address(),
            max_precision_decimals,
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
            pool_type: PairType::Xyk,
            total_fee_bps: swap_fee_bps,
            fee_recipient,
            max_allowed_slippage_bps,
            max_allowed_spread_bps,
            max_referral_bps,
        };

        save_config(&env, config);
        save_default_slippage_bps(&env, default_slippage_bps);

        utils::save_admin_old(&env, admin);
        utils::save_total_shares(&env, 0);
        utils::save_pool_balance_a(&env, 0);
        utils::save_pool_balance_b(&env, 0);

        env.storage().persistent().set(&XYK_POOL_KEY, &true);

        env.events()
            .publish(("initialize", "XYK LP token_a"), token_a);
        env.events()
            .publish(("initialize", "XYK LP token_b"), token_b);
    }

    #[allow(clippy::too_many_arguments)]
    fn provide_liquidity(
        env: Env,
        sender: Address,
        desired_a: Option<i128>,
        min_a: Option<i128>,
        desired_b: Option<i128>,
        min_b: Option<i128>,
        custom_slippage_bps: Option<i64>,
        deadline: Option<u64>,
        auto_stake: bool,
    ) {
        if let Some(deadline) = deadline {
            if env.ledger().timestamp() > deadline {
                log!(
                    env,
                    "Pool: Provide Liquidity: Transaction executed after deadline!"
                );
                panic_with_error!(env, ContractError::TransactionAfterTimestampDeadline)
            }
        }

        validate_int_parameters!(desired_a, min_a, desired_b, min_b);

        // sender needs to authorize the deposit
        sender.require_auth();

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        let config = get_config(&env);
        let pool_balance_a = utils::get_pool_balance_a(&env);
        let pool_balance_b = utils::get_pool_balance_b(&env);

        // Check if custom_slippage_bps is more than max_allowed_slippage
        if let Some(custom_slippage) = custom_slippage_bps {
            if custom_slippage > config.max_allowed_slippage_bps {
                log!(
                    &env,
                    "Pool: ProvideLiquidity: Custom slippage tolerance is more than max allowed slippage tolerance"
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
                    Decimal::bps(custom_slippage_bps.unwrap_or(get_default_slippage_bps(&env))),
                )
            }
            // None or invalid amounts are provided
            _ => {
                log!(
                    &env,
                        "Pool: ProvideLiquidity: Both tokens must be provided and must be bigger then 0!"
                );
                panic_with_error!(
                    env,
                    ContractError::ProvideLiquidityAtLeastOneTokenMustBeBiggerThenZero
                );
            }
        };
        let token_a_client = token_contract::Client::new(&env, &config.token_a);
        let token_b_client = token_contract::Client::new(&env, &config.token_b);

        // Before the transfer
        let initial_balance_a = token_a_client.balance(&env.current_contract_address());
        let initial_balance_b = token_b_client.balance(&env.current_contract_address());

        // Move tokens from client's wallet to the contract
        token_a_client.transfer(&sender, &env.current_contract_address(), &(amounts.0));
        token_b_client.transfer(&sender, &env.current_contract_address(), &(amounts.1));

        // After the transfer, get the new balances
        let final_balance_a = token_a_client.balance(&env.current_contract_address());
        let final_balance_b = token_b_client.balance(&env.current_contract_address());

        // Calculate the actual received amounts
        let actual_received_a = final_balance_a
            .checked_sub(initial_balance_a)
            .unwrap_or_else(|| {
                log!(
                    &env,
                    "Pool: Provide Liquidity: subtraction ended up in underflow/overflow for balance_a"
                );
                panic_with_error!(env, ContractError::ContractMathError);
            });

        let actual_received_b = final_balance_b
            .checked_sub(initial_balance_b)
            .unwrap_or_else(|| {
                log!(
                    &env,
                    "Pool: Provide Liquidity: subtraction ended up in underflow/overflow for balance_b"
                );
                panic_with_error!(env, ContractError::ContractMathError);
            });

        let pool_balance_a = utils::get_pool_balance_a(&env);
        let pool_balance_b = utils::get_pool_balance_b(&env);

        // Now calculate how many new pool shares to mint
        let balance_a = utils::get_balance(&env, &config.token_a);
        let balance_b = utils::get_balance(&env, &config.token_b);
        let total_shares = utils::get_total_shares(&env);

        let new_total_shares = if pool_balance_a > 0 && pool_balance_b > 0 {
            // use 10_000 multiplier to acheieve a bit bigger precision

            let shares_a = balance_a
                .checked_mul(total_shares)
                .and_then(|result| result.checked_div(pool_balance_a))
                .unwrap_or_else(|| {
                    log!(
                        env,
                        "Pool: Provide Liquidity: overflow/underflow for shares_a"
                    );
                    panic_with_error!(env, ContractError::ContractMathError);
                });
            let shares_b = balance_b
                .checked_mul(total_shares)
                .and_then(|result| result.checked_div(pool_balance_b))
                .unwrap_or_else(|| {
                    log!(
                        env,
                        "Pool: Provide Liquidity: overflow/underflow for shares_b"
                    );
                    panic_with_error!(env, ContractError::ContractMathError);
                });
            shares_a.min(shares_b)
        } else {
            // In case of empty pool, just produce X*Y shares
            let shares = amounts
                .0
                .checked_mul(amounts.1)
                .map(|product| product.sqrt())
                .unwrap_or_else(|| {
                    log!(
                        env,
                        "Pool: Provide Liquidity: multiplication overflow or invalid square root for shares"
                    );
                    panic_with_error!(env, ContractError::ContractMathError);
                });

            if MINIMUM_LIQUIDITY_AMOUNT >= shares {
                log!(env, "Pool: Provide Liquidity: Not enough liquidity!");
                panic_with_error!(env, ContractError::TotalSharesEqualZero);
            };
            // In case of an empty mint 1000 LP shares to a burner addr
            utils::mint_shares(
                &env,
                &config.share_token,
                &env.current_contract_address(),
                MINIMUM_LIQUIDITY_AMOUNT,
            );
            shares
                .checked_sub(MINIMUM_LIQUIDITY_AMOUNT)
                .unwrap_or_else(|| {
                    log!(
                        &env,
                        "Pool: Provide Liquidity: subtraction got an underflow for shares"
                    );
                    panic_with_error!(env, ContractError::ContractMathError);
                })
        };

        let shares_amount = new_total_shares
            .checked_sub(total_shares)
            .unwrap_or_else(|| {
                log!(
                    &env,
                    "Pool: Provide Liquidity: subtraction got an underflow for shares_amount"
                );
                panic_with_error!(env, ContractError::ContractMathError);
            });

        utils::mint_shares(&env, &config.share_token, &sender, shares_amount);

        if auto_stake {
            let stake_contract_client = stake_contract::Client::new(&env, &config.stake_contract);

            stake_contract_client.bond(&sender, &shares_amount);
        }

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
        // FIXM: Disable Referral struct
        // referral: Option<Referral>,
        offer_asset: Address,
        offer_amount: i128,
        ask_asset_min_amount: Option<i128>,
        max_spread_bps: Option<i64>,
        deadline: Option<u64>,
        max_allowed_fee_bps: Option<i64>,
    ) -> i128 {
        if let Some(deadline) = deadline {
            if env.ledger().timestamp() > deadline {
                log!(env, "Pool: Swap: Transaction executed after deadline!");
                panic_with_error!(env, ContractError::TransactionAfterTimestampDeadline)
            }
        }

        validate_int_parameters!(offer_amount);

        sender.require_auth();

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        do_swap(
            env,
            sender,
            // referral,
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
                    "Pool: Withdraw Liquidity: Transaction executed after deadline!"
                );
                panic_with_error!(env, ContractError::TransactionAfterTimestampDeadline)
            }
        }

        if min_a.is_negative() || min_b.is_negative() {
            log!(
                env,
                "Pool: Withdraw Liquidity: Negative value for min_a or min_b"
            );
            panic_with_error!(env, ContractError::NegativeInputProvided)
        }

        validate_int_parameters!(share_amount);

        sender.require_auth();
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        let config = get_config(&env);

        let share_token_client = token_contract::Client::new(&env, &config.share_token);
        share_token_client.transfer(&sender, &env.current_contract_address(), &share_amount);

        let pool_balance_a = utils::get_pool_balance_a(&env);
        let pool_balance_b = utils::get_pool_balance_b(&env);

        let total_shares = utils::get_total_shares(&env);

        if total_shares == 0i128 {
            log!(&env, "Pool: WithdrawLiquidity: Critical error - Total shares are equal to zero before withdrawal!");
            panic_with_error!(env, ContractError::TotalSharesEqualZero);
        }

        let share_ratio = Decimal::from_ratio(share_amount, total_shares);

        //safe math done in Decimal
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
        let admin: Address = utils::get_admin_old(&env);
        admin.require_auth();
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        let mut config = get_config(&env);

        // TODO: fix that - admin shouldn't be part of updating the struct
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
        if let Some(max_referral_bps) = max_referral_bps {
            validate_bps!(max_referral_bps);
            config.max_referral_bps = max_referral_bps;
        }

        save_config(&env, config);
    }

    fn upgrade(env: Env, new_wasm_hash: BytesN<32>, new_default_slippage_bps: i64) {
        let admin: Address = utils::get_admin_old(&env);
        admin.require_auth();

        env.deployer().update_current_contract_wasm(new_wasm_hash);
        save_default_slippage_bps(&env, new_default_slippage_bps);
    }

    // Queries

    fn query_config(env: Env) -> Config {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        get_config(&env)
    }

    fn query_share_token_address(env: Env) -> Address {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        get_config(&env).share_token
    }

    fn query_stake_contract_address(env: Env) -> Address {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);
        get_config(&env).stake_contract
    }

    fn query_pool_info(env: Env) -> PoolResponse {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);
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

    fn query_pool_info_for_factory(env: Env) -> LiquidityPoolInfo {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);
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

        LiquidityPoolInfo {
            pool_address: env.current_contract_address(),
            pool_response,
            total_fee_bps,
        }
    }

    fn simulate_swap(env: Env, offer_asset: Address, offer_amount: i128) -> SimulateSwapResponse {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);
        let config = get_config(&env);

        let pool_balance_a = utils::get_pool_balance_a(&env);
        let pool_balance_b = utils::get_pool_balance_b(&env);
        let (pool_balance_offer, pool_balance_ask) = if offer_asset == config.token_a {
            (pool_balance_a, pool_balance_b)
        } else if offer_asset == config.token_b {
            (pool_balance_b, pool_balance_a)
        } else {
            log!(&env, "Pool: Token offered to swap not found in Pool");
            panic_with_error!(env, ContractError::AssetNotInPool);
        };

        let compute_swap: ComputeSwap = compute_swap(
            &env,
            pool_balance_offer,
            pool_balance_ask,
            offer_amount,
            config.protocol_fee_rate(),
            0i64,
        );

        let total_return = compute_swap
            .return_amount
            .checked_add(compute_swap.commission_amount)
            .and_then(|partial_sum| partial_sum.checked_add(compute_swap.spread_amount))
            .unwrap_or_else(|| {
                log!(&env, "Pool: Simulate Swap: addition overflowed");
                panic_with_error!(env, ContractError::ContractMathError);
            });

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
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);
        let config = get_config(&env);

        let pool_balance_a = utils::get_pool_balance_a(&env);
        let pool_balance_b = utils::get_pool_balance_b(&env);
        let (pool_balance_offer, pool_balance_ask) = if ask_asset == config.token_b {
            (pool_balance_a, pool_balance_b)
        } else if ask_asset == config.token_a {
            (pool_balance_b, pool_balance_a)
        } else {
            log!(&env, "Pool: Token offered to swap not found in Pool");
            panic_with_error!(env, ContractError::AssetNotInPool);
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

    fn query_share(env: Env, amount: i128) -> (Asset, Asset) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);
        let pool_info = Self::query_pool_info(env);
        let total_share = pool_info.asset_lp_share.amount;
        let token_a_amount = pool_info.asset_a.amount;
        let token_b_amount = pool_info.asset_b.amount;

        let mut share_ratio = Decimal::zero();
        if total_share != 0 {
            share_ratio = Decimal::from_ratio(amount, total_share);
        }

        //safe math done in Decimal multiplication
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
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);
        utils::get_total_shares(&env)
    }

    fn migrate_admin_key(env: Env) -> Result<(), ContractError> {
        let admin = get_admin_old(&env);
        env.storage().instance().set(&ADMIN, &admin);
        Ok(())
    }

    fn propose_admin(
        env: Env,
        new_admin: Address,
        time_limit: Option<u64>,
    ) -> Result<Address, ContractError> {
        let current_admin = get_admin_old(&env);
        current_admin.require_auth();

        if current_admin == new_admin {
            log!(&env, "Trying to set new admin as new");
            panic_with_error!(&env, ContractError::SameAdmin);
        }

        env.storage().instance().set(
            &PENDING_ADMIN,
            &AdminChange {
                new_admin: new_admin.clone(),
                time_limit,
            },
        );

        env.events().publish(
            ("XYK Pool: ", "Admin replacement requested by old admin: "),
            &current_admin,
        );
        env.events()
            .publish(("XYK Pool: ", "Replace with new admin: "), &new_admin);

        Ok(new_admin)
    }

    fn revoke_admin_change(env: Env) -> Result<(), ContractError> {
        let current_admin = get_admin_old(&env);
        current_admin.require_auth();

        if !env.storage().instance().has(&PENDING_ADMIN) {
            log!(&env, "No admin change in place");
            panic_with_error!(&env, ContractError::NoAdminChangeInPlace);
        }

        env.storage().instance().remove(&PENDING_ADMIN);

        env.events()
            .publish(("XYK Pool: ", "Undo admin change: "), ());

        Ok(())
    }

    fn accept_admin(env: Env) -> Result<Address, ContractError> {
        let admin_change_info: AdminChange = env
            .storage()
            .instance()
            .get(&PENDING_ADMIN)
            .unwrap_or_else(|| {
                log!(&env, "No admin change request is in place");
                panic_with_error!(&env, ContractError::NoAdminChangeInPlace);
            });

        let pending_admin = admin_change_info.new_admin;
        pending_admin.require_auth();

        if let Some(time_limit) = admin_change_info.time_limit {
            if env.ledger().timestamp() > time_limit {
                log!(&env, "Admin change expired");
                panic_with_error!(&env, ContractError::AdminChangeExpired);
            }
        }

        env.storage().instance().remove(&PENDING_ADMIN);

        utils::save_admin_old(&env, pending_admin.clone());

        env.events()
            .publish(("XYK Pool: ", "Accepted new admin: "), &pending_admin);

        Ok(pending_admin)
    }
}

#[contractimpl]
impl LiquidityPool {
    #[allow(dead_code)]
    pub fn update(env: Env, new_wasm_hash: BytesN<32>) {
        let admin = get_admin_old(&env);
        admin.require_auth();

        env.deployer().update_current_contract_wasm(new_wasm_hash);
    }

    #[allow(dead_code)]
    pub fn query_version(env: Env) -> String {
        String::from_str(&env, env!("CARGO_PKG_VERSION"))
    }

    #[allow(dead_code)]
    //TODO: Remove after we've added the key to storage
    pub fn add_contract_name_key_to_storage(env: Env) -> Result<(), ContractError> {
        env.storage().persistent().set(&XYK_POOL_KEY, &true);
        Ok(())
    }

    #[allow(dead_code)]
    pub fn extend_all_tll(env: Env) -> Result<(), ContractError> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        env.storage().persistent().extend_ttl(
            &DEFAULT_SLIPPAGE_BPS,
            PERSISTENT_RENEWAL_THRESHOLD,
            PERSISTENT_TARGET_TTL,
        );

        env.storage().persistent().extend_ttl(
            &CONFIG,
            PERSISTENT_RENEWAL_THRESHOLD,
            PERSISTENT_TARGET_TTL,
        );

        for val in &[
            DataKey::TotalShares,
            DataKey::ReserveA,
            DataKey::ReserveB,
            DataKey::Admin,
            DataKey::Initialized,
        ] {
            env.storage().persistent().extend_ttl(
                val,
                PERSISTENT_RENEWAL_THRESHOLD,
                PERSISTENT_TARGET_TTL,
            )
        }

        env.storage().persistent().has(&XYK_POOL_KEY).then(|| {
            env.storage().persistent().extend_ttl(
                &XYK_POOL_KEY,
                PERSISTENT_RENEWAL_THRESHOLD,
                PERSISTENT_TARGET_TTL,
            )
        });

        Ok(())
    }
}

fn do_swap(
    env: Env,
    sender: Address,
    // FIXM: Disable Referral struct
    // referral: Option<Referral>,
    offer_asset: Address,
    offer_amount: i128,
    ask_asset_min_amount: Option<i128>,
    max_spread: Option<i64>,
    max_allowed_fee_bps: Option<i64>,
) -> i128 {
    let config = get_config(&env);
    // FIXM: Disable Referral struct
    // if let Some(referral) = &referral {
    //     if referral.fee > config.max_referral_bps {
    //         panic!("Pool: Swap: Trying to swap with more than the allowed referral fee");
    //     }
    // }
    if let Some(agreed_percentage) = max_allowed_fee_bps {
        if agreed_percentage < config.total_fee_bps {
            log!(
                &env,
                "Pool: do_swap: User agrees to swap at a lower percentage."
            );
            panic_with_error!(&env, ContractError::UserDeclinesPoolFee);
        }
    }

    if let Some(max_spread) = max_spread {
        if !(0..=config.max_allowed_spread_bps).contains(&max_spread) {
            log!(&env, "Pool: do_swap: max spread is out of bounds");
            panic_with_error!(&env, ContractError::InvalidBps);
        }
    }

    let max_spread = Decimal::bps(max_spread.map_or_else(|| config.max_allowed_spread_bps, |x| x));

    let pool_balance_a = utils::get_pool_balance_a(&env);
    let pool_balance_b = utils::get_pool_balance_b(&env);

    let (pool_balance_sell, pool_balance_buy) = if offer_asset == config.token_a {
        (pool_balance_a, pool_balance_b)
    } else if offer_asset == config.token_b {
        (pool_balance_b, pool_balance_a)
    } else {
        log!(&env, "Pool: Token offered to swap not found in Pool");
        panic_with_error!(env, ContractError::AssetNotInPool);
    };

    // FIXM: Disable Referral struct
    // let referral_fee_bps = match referral {
    //     Some(ref referral) => referral.clone().fee,
    //     None => 0,
    // };
    let referral_fee_bps = 0;

    // 1. We calculate the referral_fee below. If none referral fee will be 0
    let compute_swap: ComputeSwap = compute_swap(
        &env,
        pool_balance_sell,
        pool_balance_buy,
        offer_amount,
        config.protocol_fee_rate(),
        referral_fee_bps,
    );

    if let Some(ask_asset_min_amount) = ask_asset_min_amount {
        if ask_asset_min_amount > compute_swap.return_amount {
            log!(
                &env,
                "Pool: do_swap: Return amount is smaller then expected minimum amount"
            );
            panic_with_error!(&env, ContractError::SwapMinReceivedBiggerThanReturn);
        }
    }

    let total_return_amount = compute_swap
        .return_amount
        .checked_add(compute_swap.commission_amount)
        .and_then(|partial_sum| partial_sum.checked_add(compute_swap.referral_fee_amount))
        .unwrap_or_else(|| {
            log!(&env, "Pool: Do Swap: addition overflowed");
            panic_with_error!(env, ContractError::ContractMathError);
        });

    assert_max_spread(
        &env,
        max_spread,
        total_return_amount,
        compute_swap.spread_amount,
    );

    // Transfer the amount being sold to the contract
    let (sell_token, buy_token) = if offer_asset == config.clone().token_a {
        (config.clone().token_a, config.clone().token_b)
    } else {
        (config.clone().token_b, config.clone().token_a)
    };

    let sell_token_client = token_contract::Client::new(&env, &sell_token);

    // we check the balance of the transferred token for the contract prior to the transfer
    let balance_before_transfer = sell_token_client.balance(&env.current_contract_address());

    // transfer tokens to swap
    sell_token_client.transfer(&sender, &env.current_contract_address(), &offer_amount);

    // get the balance after the transfer
    let balance_after_transfer = sell_token_client.balance(&env.current_contract_address());

    // calculate how much did the contract actually got
    let actual_received_amount = balance_after_transfer
        .checked_sub(balance_before_transfer)
        .unwrap_or_else(|| {
            log!(&env, "Pool: Do Swap: Subtraction underflowed.");
            panic_with_error!(&env, ContractError::ContractMathError);
        });

    let buy_token_client = token_contract::Client::new(&env, &buy_token);

    // return swapped tokens to user
    buy_token_client.transfer(
        &env.current_contract_address(),
        &sender,
        &compute_swap.return_amount,
    );

    // send commission to fee recipient
    buy_token_client.transfer(
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
        let balance_a = pool_balance_a
            .checked_add(actual_received_amount)
            .unwrap_or_else(|| {
                log!(&env, "Pool: Do Swap: addition overflowed");
                panic_with_error!(&env, ContractError::ContractMathError)
            });

        let balance_b = pool_balance_b
            .checked_sub(compute_swap.commission_amount)
            .and_then(|partial| partial.checked_sub(compute_swap.referral_fee_amount))
            .and_then(|partial| partial.checked_sub(compute_swap.return_amount))
            .unwrap_or_else(|| {
                log!(&env, "Pool: Do Swap: subtraction underflowed");
                panic_with_error!(&env, ContractError::ContractMathError)
            });

        (balance_a, balance_b)
    } else {
        let balance_a = pool_balance_a
            .checked_sub(compute_swap.commission_amount)
            .and_then(|partial| partial.checked_sub(compute_swap.referral_fee_amount))
            .and_then(|partial| partial.checked_sub(compute_swap.return_amount))
            .unwrap_or_else(|| {
                log!(&env, "Pool: Do Swap: subtraction underflowed");
                panic_with_error!(&env, ContractError::ContractMathError)
            });

        let balance_b = pool_balance_b
            .checked_add(actual_received_amount)
            .unwrap_or_else(|| {
                log!(&env, "Pool: Do Swap: addition overflowed");
                panic_with_error!(&env, ContractError::ContractMathError)
            });

        (balance_a, balance_b)
    };
    utils::save_pool_balance_a(&env, balance_a);
    utils::save_pool_balance_b(&env, balance_b);

    env.events().publish(("swap", "sender"), sender);
    env.events().publish(("swap", "sell_token"), sell_token);
    env.events().publish(("swap", "offer_amount"), offer_amount);
    env.events()
        .publish(("swap", "actual received amount"), actual_received_amount);
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
///
// TODO: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/issues/204
#[allow(dead_code)]
fn split_deposit_based_on_pool_ratio(
    env: &Env,
    config: &Config,
    a_pool: i128,
    b_pool: i128,
    deposit: i128,
    offer_asset: &Address,
) -> (i128, i128) {
    // check if offer_asset is one of the two tokens in the pool
    if offer_asset != &config.token_a && offer_asset != &config.token_b {
        log!(&env, "Pool: Token offered to swap not found in Pool");
        panic_with_error!(env, ContractError::AssetNotInPool);
    }

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
        // Calculate middle point
        let mid = low
            .checked_add(high)
            .and_then(|sum| sum.checked_div(2))
            .unwrap_or_else(|| {
                log!(
                    &env,
                    "Pool: Split Deposit Based On Pool Ratio: overflow/underflow occured."
                );
                panic_with_error!(&env, ContractError::ContractMathError)
            });

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
        let diff = deposit.checked_sub(mid).unwrap_or_else(|| {
            log!(
                &env,
                "Pool: Split Deposit Baed On Pool Ratio: underflow occured."
            );
            panic_with_error!(&env, ContractError::ContractMathError);
        });
        let ratio = if offer_asset == &config.token_a {
            Decimal::from_ratio(ask_amount, diff)
        } else {
            Decimal::from_ratio(diff, ask_amount)
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
        log!(
            env,
            "Pool: Slippage tolerance exceeds the maximum allowed value"
        );
        panic_with_error!(&env, ContractError::SlippageInvalid);
    }

    // Calculate the limit below which the deposit-to-pool ratio must not fall for each token
    //safe math division done in Decimal
    let one_minus_slippage_tolerance = Decimal::one() - slippage_tolerance;
    let deposits: [i128; 2] = [deposits[0], deposits[1]];
    let pools: [i128; 2] = [pools[0], pools[1]];

    // Ensure each price does not change more than what the slippage tolerance allows
    //safe math done in Decimal
    if deposits[0] * pools[1] * one_minus_slippage_tolerance
        > deposits[1] * pools[0] * Decimal::one()
        || deposits[1] * pools[0] * one_minus_slippage_tolerance
            > deposits[0] * pools[1] * Decimal::one()
    {
        log!(
            &env,
            "Pool: Assert slippage tolerance: slippage tolerance violated"
        );
        panic_with_error!(&env, ContractError::SlippageInvalid);
    }
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
        log!(env, "Pool: Spread exceeds maximum allowed");
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
    env: &Env,
    offer_pool: i128,
    ask_pool: i128,
    offer_amount: i128,
    commission_rate: Decimal,
    referral_fee: i64,
) -> ComputeSwap {
    let offer_pool_as_u256 = U256::from_u128(env, convert_i128_to_u128(offer_pool));
    let ask_pool_as_u256 = U256::from_u128(env, convert_i128_to_u128(ask_pool));
    let offer_amount_as_u256 = U256::from_u128(env, convert_i128_to_u128(offer_amount));
    let commmission_rate_as_u256 =
        U256::from_u128(env, convert_i128_to_u128(commission_rate.atomics()));

    // Calculate the cross product of offer_pool and ask_pool
    let cp = offer_pool_as_u256.mul(&ask_pool_as_u256);

    // Calculate the resulting amount of ask assets after the swap
    // Return amount calculation based on the AMM model's invariant,
    // which ensures the product of the amounts of the two assets remains constant before and after a trade.
    let return_amount =
        ask_pool_as_u256.sub(&(cp.div(&offer_pool_as_u256.add(&offer_amount_as_u256))));
    // Calculate the spread amount, representing the difference between the expected and actual swap amounts
    let expected_return = offer_amount_as_u256
        .mul(&ask_pool_as_u256)
        .div(&offer_pool_as_u256);
    let spread_amount = if expected_return > return_amount {
        expected_return.sub(&return_amount)
    } else {
        U256::from_u128(env, 0)
    };

    let decimal_fractional = U256::from_u128(env, 1_000_000_000_000_000_000u128);
    let commission_amount = return_amount
        .mul(&commmission_rate_as_u256)
        .div(&decimal_fractional);

    // Deduct the commission (minus the part that goes to the protocol) from the return amount
    let return_amount = return_amount.sub(&commission_amount);

    let referral_fee_as_u256_from_bps = U256::from_u128(
        env,
        convert_i128_to_u128(Decimal::bps(referral_fee).atomics()),
    );
    let referral_fee_amount = return_amount
        .mul(&referral_fee_as_u256_from_bps)
        .div(&decimal_fractional);

    let return_amount = return_amount.sub(&referral_fee_amount);

    ComputeSwap {
        return_amount: u256_to_i128(env, return_amount),
        spread_amount: u256_to_i128(env, spread_amount),
        commission_amount: u256_to_i128(env, commission_amount),
        referral_fee_amount: u256_to_i128(env, referral_fee_amount),
    }
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
    // Calculate the cross product of offer_pool and ask_pool
    let cp: i128 = offer_pool.checked_mul(ask_pool).unwrap_or_else(|| {
        log!(
            env,
            "Pool: Compute Offer Amount: checked multiplication overflowed."
        );
        panic_with_error!(env, ContractError::ContractMathError);
    });

    // Calculate one minus the commission rate
    let one_minus_commission = Decimal::one() - commission_rate;

    // Calculate the inverse of one minus the commission rate
    let inv_one_minus_commission = Decimal::one() / one_minus_commission;

    // Calculate the resulting amount of ask assets after the swap
    let offer_amount: i128 = cp / (ask_pool - (ask_amount * inv_one_minus_commission)) - offer_pool;

    let ask_before_commission = ask_amount * inv_one_minus_commission;

    // Calculate the spread amount, representing the difference between the expected and actual swap amounts
    let spread_amount: i128 = offer_amount
        .checked_mul(ask_pool)
        .and_then(|product| product.checked_div(offer_pool))
        .and_then(|result| result.checked_sub(ask_before_commission))
        .unwrap_or_else(|| {
            log!(
                env,
                "Pool: Compute Offer Amount: overflow/underflow occured."
            );
            panic_with_error!(env, ContractError::ContractMathError)
        });

    // Calculate the commission amount
    let commission_amount: i128 = ask_before_commission * commission_rate;

    (offer_amount, spread_amount, commission_amount)
}

fn u256_to_i128(env: &Env, value: U256) -> i128 {
    value
        .to_u128()
        .and_then(|v| i128::try_from(v).ok())
        .unwrap_or_else(|| {
            log!(env, "Pool: Compute swap: Unable to convert U256 to i128");
            panic_with_error!(env, ContractError::CannotConvertU256ToI128);
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use soroban_sdk::{testutils::Address as _, Address};

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
    #[should_panic(expected = "Pool: Slippage tolerance exceeds the maximum allowed value")]
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
        // max spread of 10%, offer amount of 100k, return amount of 100k and 1 unit, spread amount of 1
        // The spread ratio is 10% which is equal to the max spread
        assert_max_spread(&env, Decimal::percent(10), 100_001, 1);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #300)")]
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
    #[should_panic(expected = "HostError: Error(Contract, #300)")]
    fn test_assert_max_spread_fail_no_belief_price_max_spread_exceeded() {
        let env = Env::default();
        // max spread of 10%, return amount of 10, spread amount of 2
        // The spread ratio is 20% which is greater than the max spread
        assert_max_spread(&env, Decimal::percent(10), 10, 2);
    }

    #[test]
    fn test_compute_swap_pass() {
        let env = Env::default();
        let result = compute_swap(&env, 1000, 2000, 100, Decimal::percent(10), 0i64); // 10% commission rate
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
        let env = Env::default();
        let result = compute_swap(&env, 1000, 2000, 100, Decimal::percent(10), 1_500i64);
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
        let env = Env::default();
        let result = compute_swap(&env, 1000, 2000, 100, Decimal::one(), 0i64); // 100% commission rate should lead to return_amount being 0
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
        let env = Env::default();
        let offer_pool = 1000000;
        let ask_pool = 1000000;
        let commission_rate = Decimal::percent(10);
        let ask_amount = 1000;

        let result = compute_offer_amount(&env, offer_pool, ask_pool, ask_amount, commission_rate);

        // Test that the offer amount is less than the original pool size, due to commission
        assert!(result.0 < offer_pool);

        // Test that the spread amount is non-negative
        assert!(result.1 >= 0);

        // Test that the commission amount is exactly 10% of the offer amount
        assert_eq!(result.2, result.0 * Decimal::percent(10));
    }

    #[should_panic(expected = "Pool: Token offered to swap not found in Pool")]
    #[test]
    fn should_panic_when_splitting_non_existent_token() {
        let env = Env::default();
        let config = &Config {
            token_a: Address::generate(&env),
            token_b: Address::generate(&env),
            share_token: Address::generate(&env),
            stake_contract: Address::generate(&env),
            pool_type: PairType::Xyk,
            total_fee_bps: 0i64,
            fee_recipient: Address::generate(&env),
            max_allowed_slippage_bps: 100i64,
            max_allowed_spread_bps: 100i64,
            max_referral_bps: 1_000i64,
        };
        split_deposit_based_on_pool_ratio(&env, config, 100, 100, 100, &Address::generate(&env));
    }

    #[test]
    fn assert_slippage_tolerance_with_none_as_tolerance() {
        let env = Env::default();

        // assert slippage tolerance with None as tolerance should pass as well
        assert_slippage_tolerance(&env, None::<i64>, &[10, 20], &[30, 60], Decimal::bps(5_000));
    }

    #[test]
    fn convert_u256_to_i128() {
        let env = Env::default();
        let u256_value = U256::from_u128(&env, 1_000_000_000_000_000);
        let converted_to_i128 = u256_to_i128(&env, u256_value);

        assert_eq!(converted_to_i128, 1_000_000_000_000_000i128);
    }

    #[test]
    #[should_panic(expected = "Pool: Compute swap: Unable to convert U256 to i128")]
    fn convert_u256_to_i128_should_panic_when_og_value_outside_the_i128_max_range() {
        let env = Env::default();

        // using `u128::MAX`, as this is larger than `i128::MAX`
        let u256_value = U256::from_u128(&env, u128::MAX);
        u256_to_i128(&env, u256_value);
    }
}
