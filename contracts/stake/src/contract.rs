use phoenix::ttl::{INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL};
use phoenix::utils::AdminChange;
use soroban_decimal::Decimal;
use soroban_sdk::{
    contract, contractimpl, contractmeta, log, panic_with_error, vec, Address, BytesN, Env, String,
    Vec,
};

use crate::distribution::{calc_power, calculate_pending_rewards_deprecated};
use crate::storage::PENDING_ADMIN;
use crate::TOKEN_PER_POWER;
use crate::{
    distribution::{
        calculate_annualized_payout, get_distribution, get_reward_curve, get_withdraw_adjustment,
        save_distribution, save_reward_curve, save_withdraw_adjustment, update_rewards,
        withdrawable_rewards, Distribution, SHARES_SHIFT,
    },
    error::ContractError,
    msg::{
        AnnualizedReward, AnnualizedRewardsResponse, ConfigResponse, StakedResponse,
        WithdrawableReward, WithdrawableRewardsResponse,
    },
    storage::{
        get_config, get_stakes, save_config, save_stakes,
        utils::{
            self, add_distribution, get_admin_old, get_distributions, get_total_staked_counter,
            is_initialized, set_initialized,
        },
        Config, Stake, STAKE_KEY,
    },
    token_contract,
};
use curve::Curve;

// Metadata that is added on to the WASM custom section
contractmeta!(
    key = "Description",
    val = "Phoenix Protocol LP Share token staking"
);

#[contract]
pub struct Staking;

pub trait StakingTrait {
    // Sets the token contract addresses for this pool
    #[allow(clippy::too_many_arguments)]
    fn initialize(
        env: Env,
        admin: Address,
        lp_token: Address,
        min_bond: i128,
        min_reward: i128,
        manager: Address,
        owner: Address,
        max_complexity: u32,
    );

    fn bond(env: Env, sender: Address, tokens: i128);

    fn unbond(env: Env, sender: Address, stake_amount: i128, stake_timestamp: u64);

    fn unbond_deprecated(env: Env, sender: Address, stake_amount: i128, stake_timestamp: u64);

    fn create_distribution_flow(env: Env, sender: Address, asset: Address);

    fn distribute_rewards(env: Env);

    fn withdraw_rewards(env: Env, sender: Address);

    fn withdraw_rewards_deprecated(env: Env, sender: Address);

    fn fund_distribution(
        env: Env,
        sender: Address,
        start_time: u64,
        distribution_duration: u64,
        token_address: Address,
        token_amount: i128,
    );

    // QUERIES

    fn query_config(env: Env) -> ConfigResponse;

    fn query_admin(env: Env) -> Address;

    fn query_staked(env: Env, address: Address) -> StakedResponse;

    fn query_total_staked(env: Env) -> i128;

    fn query_annualized_rewards(env: Env) -> AnnualizedRewardsResponse;

    fn query_withdrawable_rewards(env: Env, address: Address) -> WithdrawableRewardsResponse;

    fn query_withdrawable_rewards_dep(env: Env, address: Address) -> WithdrawableRewardsResponse;

    fn update_config(
        env: Env,
        lp_token: Option<Address>,
        min_bond: Option<i128>,
        min_reward: Option<i128>,
        manager: Option<Address>,
        owner: Option<Address>,
        max_complexity: Option<u32>,
    ) -> Result<Config, ContractError>;

    fn update_admin(env: Env, new_admin: Address) -> Result<Address, ContractError>;

    fn query_distributed_rewards(env: Env, asset: Address) -> u128;

    fn query_undistributed_rewards(env: Env, asset: Address) -> u128;

    fn propose_admin(
        env: Env,
        new_admin: Address,
        time_limit: Option<u64>,
    ) -> Result<Address, ContractError>;

    fn revoke_admin_change(env: Env) -> Result<(), ContractError>;

    fn accept_admin(env: Env) -> Result<Address, ContractError>;
}

#[contractimpl]
impl StakingTrait for Staking {
    #[allow(clippy::too_many_arguments)]
    fn initialize(
        env: Env,
        admin: Address,
        lp_token: Address,
        min_bond: i128,
        min_reward: i128,
        manager: Address,
        owner: Address,
        max_complexity: u32,
    ) {
        if is_initialized(&env) {
            log!(
                &env,
                "Stake: Initialize: initializing contract twice is not allowed"
            );
            panic_with_error!(&env, ContractError::AlreadyInitialized);
        }

        set_initialized(&env);

        if min_bond <= 0 {
            log!(
                &env,
                "Stake: initialize: Minimum amount of lp share tokens to bond can not be smaller or equal to 0"
            );
            panic_with_error!(&env, ContractError::InvalidMinBond);
        }
        if min_reward <= 0 {
            log!(&env, "Stake: initialize: min_reward must be bigger than 0!");
            panic_with_error!(&env, ContractError::InvalidMinReward);
        }

        if max_complexity == 0 {
            log!(
                &env,
                "Stake: initialize: max_complexity must be bigger than 0!"
            );
            panic_with_error!(&env, ContractError::InvalidMaxComplexity);
        }

        env.events()
            .publish(("initialize", "LP Share token staking contract"), &lp_token);

        let config = Config {
            lp_token,
            min_bond,
            min_reward,
            manager,
            owner,
            max_complexity,
        };
        save_config(&env, config);

        utils::save_admin_old(&env, &admin);
        utils::init_total_staked(&env);

        env.storage().persistent().set(&STAKE_KEY, &true);

        env.events()
            .publish(("Stake", "Initialized with admin: "), admin);
    }

    fn bond(env: Env, sender: Address, tokens: i128) {
        sender.require_auth();
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        let ledger = env.ledger();
        let config = get_config(&env);

        if tokens < config.min_bond {
            log!(
                &env,
                "Stake: Bond: Trying to stake less than minimum required"
            );
            panic_with_error!(&env, ContractError::InvalidBond);
        }

        let lp_token_client = token_contract::Client::new(&env, &config.lp_token);
        lp_token_client.transfer(&sender, &env.current_contract_address(), &tokens);

        let mut stakes = get_stakes(&env, &sender);
        let stake = Stake {
            stake: tokens,
            stake_timestamp: ledger.timestamp(),
        };

        stakes.total_stake = stakes.total_stake.checked_add(tokens).unwrap_or_else(|| {
            log!(&env, "Stake: Bond: overflow occured.");
            panic_with_error!(&env, ContractError::ContractMathError);
        });
        // TODO: Discuss: Add implementation to add stake if another is present in +-24h timestamp to avoid
        // creating multiple stakes the same day

        for distribution_address in get_distributions(&env) {
            let mut distribution = get_distribution(&env, &distribution_address);
            let stakes: i128 = get_stakes(&env, &sender).total_stake;
            let old_power = calc_power(&config, stakes, Decimal::one(), TOKEN_PER_POWER); // while bonding we use Decimal::one()
            let stakes_sum = stakes.checked_add(tokens).unwrap_or_else(|| {
                log!(&env, "Stake: Bond: Overflow occured.");
                panic_with_error!(&env, ContractError::ContractMathError);
            });
            let new_power = calc_power(&config, stakes_sum, Decimal::one(), TOKEN_PER_POWER);
            update_rewards(
                &env,
                &sender,
                &distribution_address,
                &mut distribution,
                old_power,
                new_power,
            );
        }

        stakes.stakes.push_back(stake);
        save_stakes(&env, &sender, &stakes);
        utils::increase_total_staked(&env, &tokens);

        env.events().publish(("bond", "user"), &sender);
        env.events().publish(("bond", "token"), &config.lp_token);
        env.events().publish(("bond", "amount"), tokens);
    }

    fn unbond(env: Env, sender: Address, stake_amount: i128, stake_timestamp: u64) {
        sender.require_auth();
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        let config = get_config(&env);

        // check for rewards and withdraw them
        let found_rewards: WithdrawableRewardsResponse =
            Self::query_withdrawable_rewards(env.clone(), sender.clone());

        if !found_rewards.rewards.is_empty() {
            Self::withdraw_rewards(env.clone(), sender.clone());
        }

        for distribution_address in get_distributions(&env) {
            let mut distribution = get_distribution(&env, &distribution_address);
            let stakes = get_stakes(&env, &sender).total_stake;
            let old_power = calc_power(&config, stakes, Decimal::one(), TOKEN_PER_POWER); // while bonding we use Decimal::one()
            let stakes_diff = stakes.checked_sub(stake_amount).unwrap_or_else(|| {
                log!(&env, "Stake: Unbond: underflow occured.");
                panic_with_error!(&env, ContractError::ContractMathError);
            });
            let new_power = calc_power(&config, stakes_diff, Decimal::one(), TOKEN_PER_POWER);
            update_rewards(
                &env,
                &sender,
                &distribution_address,
                &mut distribution,
                old_power,
                new_power,
            );
        }

        let mut stakes = get_stakes(&env, &sender);
        remove_stake(&env, &mut stakes.stakes, stake_amount, stake_timestamp);
        stakes.total_stake = stakes
            .total_stake
            .checked_sub(stake_amount)
            .unwrap_or_else(|| {
                log!(&env, "Stake: Unbond: Underflow occured.");
                panic_with_error!(&env, ContractError::ContractMathError);
            });

        let lp_token_client = token_contract::Client::new(&env, &config.lp_token);
        lp_token_client.transfer(&env.current_contract_address(), &sender, &stake_amount);

        save_stakes(&env, &sender, &stakes);
        utils::decrease_total_staked(&env, &stake_amount);

        env.events().publish(("unbond", "user"), &sender);
        env.events().publish(("unbond", "token"), &config.lp_token);
        env.events().publish(("unbond", "amount"), stake_amount);
    }

    fn unbond_deprecated(env: Env, sender: Address, stake_amount: i128, stake_timestamp: u64) {
        sender.require_auth();

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        let config = get_config(&env);

        let mut stakes = get_stakes(&env, &sender);

        remove_stake(&env, &mut stakes.stakes, stake_amount, stake_timestamp);
        stakes.total_stake = stakes
            .total_stake
            .checked_sub(stake_amount)
            .unwrap_or_else(|| {
                log!(&env, "Stake: Unbond: underflow occured.");
                panic_with_error!(&env, ContractError::ContractMathError);
            });

        let lp_token_client = token_contract::Client::new(&env, &config.lp_token);
        lp_token_client.transfer(&env.current_contract_address(), &sender, &stake_amount);

        save_stakes(&env, &sender, &stakes);
        utils::decrease_total_staked(&env, &stake_amount);

        env.events().publish(("unbond", "user"), &sender);
        env.events().publish(("unbond", "token"), &config.lp_token);
        env.events().publish(("unbond", "amount"), stake_amount);
    }

    fn create_distribution_flow(env: Env, sender: Address, asset: Address) {
        sender.require_auth();
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        let manager = get_config(&env).manager;
        let owner = get_config(&env).owner;
        if sender != manager && sender != owner {
            log!(env, "Stake: create distribution: Non-authorized creation!");
            panic_with_error!(&env, ContractError::Unauthorized);
        }

        let distribution = Distribution {
            shares_per_point: 1u128,
            shares_leftover: 0u64,
            distributed_total: 0u128,
            withdrawable_total: 0u128,
            max_bonus_bps: 0u64,
            bonus_per_day_bps: 0u64,
        };

        let reward_token_client = token_contract::Client::new(&env, &asset);
        // add distribution to the vector of distributions
        add_distribution(&env, &reward_token_client.address);
        save_distribution(&env, &reward_token_client.address, &distribution);
        // Create the default reward distribution curve which is just a flat 0 const
        save_reward_curve(&env, asset, &Curve::Constant(0));

        env.events().publish(
            ("create_distribution_flow", "asset"),
            &reward_token_client.address,
        );
    }

    fn distribute_rewards(env: Env) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        let total_staked_amount = get_total_staked_counter(&env);
        let total_rewards_power = calc_power(
            &get_config(&env),
            total_staked_amount,
            Decimal::one(),
            TOKEN_PER_POWER,
        ) as u128;

        if total_rewards_power == 0 {
            log!(&env, "Stake: No rewards to distribute!");
            return;
        }
        for distribution_address in get_distributions(&env) {
            let mut distribution = get_distribution(&env, &distribution_address);
            let withdrawable = distribution.withdrawable_total;

            let reward_token_client = token_contract::Client::new(&env, &distribution_address);
            // Undistributed rewards are simply all tokens left on the contract
            let undistributed_rewards =
                reward_token_client.balance(&env.current_contract_address()) as u128;

            let curve = get_reward_curve(&env, &distribution_address).unwrap_or_else(|| {
                log!(&env, "Stake: Distribute reward: Not reward curve exists, probably distribution haven't been created");
                panic_with_error!(&env, ContractError::RewardCurveDoesNotExist);
            });

            // Calculate how much we have received since the last time Distributed was called,
            // including only the reward config amount that is eligible for distribution.
            // This is the amount we will distribute to all mem
            let amount = undistributed_rewards
                .checked_sub(withdrawable)
                .and_then(|diff| diff.checked_sub(curve.value(env.ledger().timestamp())))
                .unwrap_or_else(|| {
                    log!(&env, "Stake: Distribute Rewards: Underflow occured.");
                    panic_with_error!(&env, ContractError::ContractMathError);
                });

            if amount == 0 {
                continue;
            }

            let leftover: u128 = distribution.shares_leftover.into();
            let shifted_left = amount.checked_shl(SHARES_SHIFT.into()).unwrap_or_else(|| {
                log!(&env, "Stake: Distribute Rewards: Overflow occured.");
                panic_with_error!(&env, ContractError::ContractMathError);
            });

            let points = shifted_left.checked_add(leftover).unwrap_or_else(|| {
                log!(&env, "Stake: Distribute Rewards: Overflow occured.");
                panic_with_error!(&env, ContractError::ContractMathError);
            });
            let points_per_share = points.checked_div(total_rewards_power).unwrap_or_else(|| {
                log!(&env, "Stake: Distribute Rewards: Overflow occured.");
                panic_with_error!(&env, ContractError::ContractMathError);
            });
            distribution.shares_leftover = (points % total_rewards_power) as u64;

            // Everything goes back to 128-bits/16-bytes
            // Full amount is added here to total withdrawable, as it should not be considered on its own
            // on future distributions - even if because of calculation offsets it is not fully
            // distributed, the error is handled by leftover.
            distribution.shares_per_point = distribution
                .shares_per_point
                .checked_add(points_per_share)
                .unwrap_or_else(|| {
                    log!(&env, "Stake: Distribute Rewards: Overflow occured.");
                    panic_with_error!(&env, ContractError::ContractMathError);
                });
            distribution.distributed_total = distribution
                .distributed_total
                .checked_add(amount)
                .unwrap_or_else(|| {
                    log!(&env, "Stake: Distribute Rewards: Overflow occured.");
                    panic_with_error!(&env, ContractError::ContractMathError);
                });
            distribution.withdrawable_total = distribution
                .withdrawable_total
                .checked_add(amount)
                .unwrap_or_else(|| {
                    log!(&env, "Stake: Distribute Rewards: Overflow occured.");
                    panic_with_error!(&env, ContractError::ContractMathError);
                });

            save_distribution(&env, &distribution_address, &distribution);

            env.events().publish(
                ("distribute_rewards", "asset"),
                &reward_token_client.address,
            );
            env.events()
                .publish(("distribute_rewards", "amount"), amount);
        }
    }

    fn withdraw_rewards(env: Env, sender: Address) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        env.events().publish(("withdraw_rewards", "user"), &sender);
        let config = get_config(&env);

        for distribution_address in get_distributions(&env) {
            // get distribution data for the given reward
            let mut distribution = get_distribution(&env, &distribution_address);
            // get withdraw adjustment for the given distribution
            let mut withdraw_adjustment =
                get_withdraw_adjustment(&env, &sender, &distribution_address);
            // calculate current reward amount given the distribution and subtracting withdraw
            // adjustments
            let reward_amount =
                withdrawable_rewards(&env, &sender, &distribution, &withdraw_adjustment, &config);

            if reward_amount == 0 {
                continue;
            }
            withdraw_adjustment.withdrawn_rewards = withdraw_adjustment
                .withdrawn_rewards
                .checked_add(reward_amount)
                .unwrap_or_else(|| {
                    log!(&env, "Stake: Withdraw Rewards: Overflow occured.");
                    panic_with_error!(&env, ContractError::ContractMathError);
                });
            distribution.withdrawable_total = distribution
                .withdrawable_total
                .checked_sub(reward_amount)
                .unwrap_or_else(|| {
                    log!(&env, "Stake: Withdraw Rewards: Underflow occured.");
                    panic_with_error!(&env, ContractError::ContractMathError);
                });

            save_distribution(&env, &distribution_address, &distribution);
            save_withdraw_adjustment(&env, &sender, &distribution_address, &withdraw_adjustment);

            let reward_token_client = token_contract::Client::new(&env, &distribution_address);
            reward_token_client.transfer(
                &env.current_contract_address(),
                &sender,
                &(reward_amount as i128),
            );

            env.events().publish(
                ("withdraw_rewards", "reward_token"),
                &reward_token_client.address,
            );
            env.events()
                .publish(("withdraw_rewards", "reward_amount"), reward_amount);
        }
    }

    fn withdraw_rewards_deprecated(env: Env, sender: Address) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        let mut stakes = get_stakes(&env, &sender);

        for asset in get_distributions(&env) {
            let pending_reward = calculate_pending_rewards_deprecated(&env, &asset, &stakes);
            env.events()
                .publish(("withdraw_rewards", "reward_token"), &asset);

            token_contract::Client::new(&env, &asset).transfer(
                &env.current_contract_address(),
                &sender,
                &pending_reward,
            );
        }

        stakes.last_reward_time = env.ledger().timestamp();
        save_stakes(&env, &sender, &stakes);
    }

    fn fund_distribution(
        env: Env,
        sender: Address,
        start_time: u64,
        distribution_duration: u64,
        token_address: Address,
        token_amount: i128,
    ) {
        sender.require_auth();
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        // Load previous reward curve; it must exist if the distribution exists
        // In case of first time funding, it will be a constant 0 curve
        let previous_reward_curve = get_reward_curve(&env, &token_address).expect("Stake: Fund distribution: Not reward curve exists, probably distribution haven't been created");
        let max_complexity = get_config(&env).max_complexity;

        let current_time = env.ledger().timestamp();
        if start_time < current_time {
            log!(
                &env,
                "Stake: Fund distribution: Fund distribution start time is too early"
            );
            panic_with_error!(&env, ContractError::InvalidTime);
        }

        let config = get_config(&env);
        if config.min_reward > token_amount {
            log!(
                &env,
                "Stake: Fund distribution: minimum reward amount not reached",
            );
            panic_with_error!(&env, ContractError::MinRewardNotEnough);
        }

        // transfer tokens to fund distribution
        let reward_token_client = token_contract::Client::new(&env, &token_address);
        reward_token_client.transfer(&sender, &env.current_contract_address(), &token_amount);

        let end_time = current_time
            .checked_add(distribution_duration)
            .unwrap_or_else(|| {
                log!(&env, "Stake: Fund Distribution: Overflow occured.");
                panic_with_error!(&env, ContractError::ContractMathError);
            });
        // define a distribution curve starting at start_time with token_amount of tokens
        // and ending at end_time with 0 tokens
        let new_reward_distribution =
            Curve::saturating_linear((start_time, token_amount as u128), (end_time, 0));

        // Validate the the curve locks at most the amount provided and
        // also fully unlocks all rewards sent
        let (min, max) = new_reward_distribution.range();
        if min != 0 || max > token_amount as u128 {
            log!(&env, "Stake: Fund distribution: Rewards validation failed");
            panic_with_error!(&env, ContractError::RewardsInvalid);
        }

        let new_reward_curve: Curve;
        // if the previous reward curve has ended, we can just use the new curve
        match previous_reward_curve.end() {
            Some(end_distribution_timestamp) if end_distribution_timestamp < current_time => {
                new_reward_curve = new_reward_distribution;
            }
            _ => {
                // if the previous distribution is still ongoing, we need to combine the two
                new_reward_curve = previous_reward_curve.combine(&env, &new_reward_distribution);
                new_reward_curve
                    .validate_complexity(max_complexity)
                    .unwrap_or_else(|_| {
                        log!(
                            &env,
                            "Stake: Fund distribution: Curve complexity validation failed"
                        );
                        panic_with_error!(&env, ContractError::InvalidMaxComplexity);
                    });
            }
        }

        save_reward_curve(&env, token_address.clone(), &new_reward_curve);

        env.events()
            .publish(("fund_distribution", "asset"), &token_address);
        env.events()
            .publish(("fund_distribution", "amount"), token_amount);
        env.events()
            .publish(("fund_distribution", "start_time"), start_time);
        env.events()
            .publish(("fund_distribution", "end_time"), end_time);
    }

    // QUERIES

    fn query_config(env: Env) -> ConfigResponse {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);
        ConfigResponse {
            config: get_config(&env),
        }
    }

    fn query_admin(env: Env) -> Address {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);
        get_admin_old(&env)
    }

    fn query_staked(env: Env, address: Address) -> StakedResponse {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);
        StakedResponse {
            stakes: get_stakes(&env, &address).stakes,
        }
    }

    fn query_total_staked(env: Env) -> i128 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);
        get_total_staked_counter(&env)
    }

    fn query_annualized_rewards(env: Env) -> AnnualizedRewardsResponse {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);
        let now = env.ledger().timestamp();
        let mut aprs = vec![&env];
        let config = get_config(&env);
        let total_stake_amount = get_total_staked_counter(&env);

        for distribution_address in get_distributions(&env) {
            let total_stake_power =
                calc_power(&config, total_stake_amount, Decimal::one(), TOKEN_PER_POWER);
            if total_stake_power == 0 {
                aprs.push_back(AnnualizedReward {
                    asset: distribution_address.clone(),
                    amount: String::from_str(&env, "0"),
                });
                continue;
            }

            // get distribution data for the given reward
            let distribution = get_distribution(&env, &distribution_address);
            let curve = get_reward_curve(&env, &distribution_address);
            let annualized_payout = calculate_annualized_payout(curve, now);
            let apr = annualized_payout
                / (total_stake_power as u128 * distribution.shares_per_point) as i128;

            aprs.push_back(AnnualizedReward {
                asset: distribution_address.clone(),
                amount: apr.to_string(&env),
            });
        }

        AnnualizedRewardsResponse { rewards: aprs }
    }

    fn query_withdrawable_rewards(env: Env, user: Address) -> WithdrawableRewardsResponse {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        let config = get_config(&env);
        // iterate over all distributions and calculate withdrawable rewards
        let mut rewards = vec![&env];
        for distribution_address in get_distributions(&env) {
            // get distribution data for the given reward
            let distribution = get_distribution(&env, &distribution_address);
            // get withdraw adjustment for the given distribution
            let withdraw_adjustment = get_withdraw_adjustment(&env, &user, &distribution_address);
            // calculate current reward amount given the distribution and subtracting withdraw
            // adjustments
            let reward_amount =
                withdrawable_rewards(&env, &user, &distribution, &withdraw_adjustment, &config);
            rewards.push_back(WithdrawableReward {
                reward_address: distribution_address,
                reward_amount,
            });
        }

        WithdrawableRewardsResponse { rewards }
    }

    fn query_withdrawable_rewards_dep(env: Env, user: Address) -> WithdrawableRewardsResponse {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);
        let stakes = get_stakes(&env, &user);
        // iterate over all distributions and calculate withdrawable rewards
        let mut rewards = vec![&env];
        for asset in get_distributions(&env) {
            let pending_reward = calculate_pending_rewards_deprecated(&env, &asset, &stakes);

            rewards.push_back(WithdrawableReward {
                reward_address: asset,
                reward_amount: pending_reward as u128,
            });
        }

        WithdrawableRewardsResponse { rewards }
    }

    fn query_distributed_rewards(env: Env, asset: Address) -> u128 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);
        let distribution = get_distribution(&env, &asset);
        distribution.distributed_total
    }

    fn update_config(
        env: Env,
        lp_token: Option<Address>,
        min_bond: Option<i128>,
        min_reward: Option<i128>,
        manager: Option<Address>,
        owner: Option<Address>,
        max_complexity: Option<u32>,
    ) -> Result<Config, ContractError> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        let admin = get_admin(&env);
        admin.require_auth();

        let mut config = get_config(&env);

        if let Some(lp_token) = lp_token {
            config.lp_token = lp_token;
        }

        if let Some(min_bond) = min_bond {
            if min_bond <= 0 {
                log!(
                &env,
                "Stake: initialize: Minimum amount of lp share tokens to bond can not be smaller or equal to 0"
            );
                panic_with_error!(&env, ContractError::InvalidMinBond);
            }
            config.min_bond = min_bond
        }

        if let Some(min_reward) = min_reward {
            if min_reward <= 0 {
                log!(&env, "Stake: initialize: min_reward must be bigger than 0!");
                panic_with_error!(&env, ContractError::InvalidMinReward);
            }
            config.min_reward = min_reward
        }

        if let Some(manager) = manager {
            config.manager = manager;
        }

        if let Some(owner) = owner {
            config.owner = owner;
        }

        if let Some(max_complexity) = max_complexity {
            if max_complexity == 0 {
                log!(
                    &env,
                    "Stake: initialize: max_complexity must be bigger than 0!"
                );
                panic_with_error!(&env, ContractError::InvalidMaxComplexity);
            }
            config.max_complexity = max_complexity
        }

        save_config(&env, config.clone());

        Ok(config)
    }

    fn update_admin(env: Env, new_admin: Address) -> Result<Address, ContractError> {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        let admin = get_admin(&env);

        admin.require_auth();

        utils::save_admin(&env, &new_admin);

        Ok(new_admin)
    }

    // fn query_distributed_rewards(env: Env, asset: Address) -> u128 {
    //     let staking_rewards = find_stake_rewards_by_asset(&env, &asset).unwrap();
    //     let unds_rew_fn_arg: Val = asset.into_val(&env);
    //     let ret: u128 = env.invoke_contract(
    //         &staking_rewards,
    //         &Symbol::new(&env, "query_distributed_reward"),
    //         vec![&env, unds_rew_fn_arg],
    //     );
    //     ret
    // }

    fn query_undistributed_rewards(env: Env, asset: Address) -> u128 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);
        let distribution = get_distribution(&env, &asset);
        let reward_token_client = token_contract::Client::new(&env, &asset);
        let reward_token_balance =
            reward_token_client.balance(&env.current_contract_address()) as u128;

        reward_token_balance
            .checked_sub(distribution.withdrawable_total)
            .unwrap_or_else(|| {
                log!(
                    &env,
                    "Stake: Query Undistributed Rewards: underflow occured."
                );
                panic_with_error!(&env, ContractError::ContractMathError);
            })
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
            ("Stake: ", "Admin replacement requested by old admin: "),
            &current_admin,
        );
        env.events()
            .publish(("Stake: ", "Replace with new admin: "), &new_admin);

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

        env.events().publish(("Stake: ", "Undo admin change: "), ());

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

        utils::save_admin_old(&env, &pending_admin);

        env.events()
            .publish(("Stake: ", "Accepted new admin: "), &pending_admin);

        Ok(pending_admin)
    }
}

#[contractimpl]
impl Staking {
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

    //TODO: Remove after we've added the key to storage
    #[allow(dead_code)]
    pub fn add_new_key_to_storage(env: Env) -> Result<(), ContractError> {
        env.storage().persistent().set(&STAKE_KEY, &true);
        Ok(())
    }

    #[allow(dead_code)]
    pub fn migrate_distributions(env: Env) {
        let distributions = get_distributions(&env);

        distributions.iter().for_each(|distribution_addr| {
            save_distribution(
                &env,
                &distribution_addr,
                &Distribution {
                    shares_per_point: 1u128,
                    shares_leftover: 0u64,
                    distributed_total: 0u128,
                    withdrawable_total: 0u128,
                    max_bonus_bps: 0u64,
                    bonus_per_day_bps: 0u64,
                },
            );
            save_reward_curve(&env, distribution_addr, &Curve::Constant(0));
        })
    }
}

// Function to remove a stake from the vector
fn remove_stake(env: &Env, stakes: &mut Vec<Stake>, stake: i128, stake_timestamp: u64) {
    // Find the index of the stake that matches the given stake and stake_timestamp
    if let Some(index) = stakes
        .iter()
        .position(|s| s.stake == stake && s.stake_timestamp == stake_timestamp)
    {
        // Remove the stake at the found index
        stakes.remove(index as u32);
    } else {
        // Stake not found, return an error
        log!(&env, "Stake: Remove stake: Stake not found");
        panic_with_error!(&env, ContractError::StakeNotFound);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::vec;

    #[test]
    fn test_remove_stake_success() {
        let env = Env::default();
        let mut stakes = vec![
            &env,
            Stake {
                stake: 100,
                stake_timestamp: 1,
            },
            Stake {
                stake: 200,
                stake_timestamp: 2,
            },
            Stake {
                stake: 150,
                stake_timestamp: 3,
            },
        ];

        let stake_to_remove = 200;
        let stake_timestamp_to_remove = 2;

        // Check that the stake is removed successfully
        remove_stake(
            &env,
            &mut stakes,
            stake_to_remove,
            stake_timestamp_to_remove,
        );

        // Check that the stake is no longer in the vector
        assert_eq!(
            stakes,
            vec![
                &env,
                Stake {
                    stake: 100,
                    stake_timestamp: 1
                },
                Stake {
                    stake: 150,
                    stake_timestamp: 3
                },
            ]
        );
    }

    #[test]
    #[should_panic(expected = "Stake: Remove stake: Stake not found")]
    fn test_remove_stake_not_found_case1() {
        let env = Env::default();
        let mut stakes = vec![
            &env,
            Stake {
                stake: 100,
                stake_timestamp: 1,
            },
            Stake {
                stake: 200,
                stake_timestamp: 2,
            },
            Stake {
                stake: 150,
                stake_timestamp: 3,
            },
        ];

        remove_stake(&env, &mut stakes, 100, 2);
    }

    #[test]
    #[should_panic(expected = "Stake: Remove stake: Stake not found")]
    fn test_remove_stake_not_found_case2() {
        let env = Env::default();
        let mut stakes = vec![
            &env,
            Stake {
                stake: 100,
                stake_timestamp: 1,
            },
            Stake {
                stake: 200,
                stake_timestamp: 2,
            },
            Stake {
                stake: 150,
                stake_timestamp: 3,
            },
        ];

        remove_stake(&env, &mut stakes, 200, 1);
    }

    #[test]
    #[should_panic(expected = "Stake: Remove stake: Stake not found")]
    fn test_remove_stake_not_found_case3() {
        let env = Env::default();
        let mut stakes = vec![
            &env,
            Stake {
                stake: 100,
                stake_timestamp: 1,
            },
            Stake {
                stake: 200,
                stake_timestamp: 2,
            },
            Stake {
                stake: 150,
                stake_timestamp: 3,
            },
        ];

        remove_stake(&env, &mut stakes, 150, 1);
    }
}
