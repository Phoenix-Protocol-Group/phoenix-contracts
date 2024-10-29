use phoenix::ttl::{INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD};
use phoenix::utils::{convert_i128_to_u128, convert_u128_to_i128};
use soroban_decimal::Decimal;
use soroban_sdk::{
    contract, contractimpl, contractmeta, log, panic_with_error, Address, BytesN, Env, String,
};

use crate::distribution::calc_power;
use crate::TOKEN_PER_POWER;
use crate::{
    distribution::{
        calc_withdraw_power, calculate_annualized_payout, get_distribution, get_reward_curve,
        get_withdraw_adjustment, save_distribution, save_reward_curve, save_withdraw_adjustment,
        update_rewards, withdrawable_rewards, Distribution, SHARES_SHIFT,
    },
    error::ContractError,
    msg::{AnnualizedRewardResponse, ConfigResponse, WithdrawableRewardResponse},
    storage::{
        get_config, save_config,
        utils::{self, get_admin, is_initialized, set_initialized},
        BondingInfo, Config,
    },
    token_contract,
};
use curve::Curve;

// Metadata that is added on to the WASM custom section
contractmeta!(
    key = "Description",
    val = "Phoenix Protocol staking rewards distribution"
);

#[contract]
pub struct StakingRewards;

#[allow(dead_code)]
pub trait StakingRewardsTrait {
    // Sets the token contract addresses for this pool
    #[allow(clippy::too_many_arguments)]
    fn initialize(
        env: Env,
        admin: Address,
        staking_contract: Address,
        reward_token: Address,
        max_complexity: u32,
        min_reward: i128,
        min_bond: i128,
    );

    fn add_user(env: Env, user: Address, stakes: BondingInfo);

    fn calculate_bond(env: Env, sender: Address, stakes: BondingInfo);

    fn calculate_unbond(env: Env, sender: Address, stakes: BondingInfo, removed_stake: i128);

    fn distribute_rewards(env: Env, total_staked_amount: i128);

    fn withdraw_rewards(env: Env, sender: Address, stakes: BondingInfo);

    fn fund_distribution(env: Env, start_time: u64, distribution_duration: u64, token_amount: i128);

    // QUERIES

    fn query_config(env: Env) -> ConfigResponse;

    fn query_admin(env: Env) -> Address;

    fn query_annualized_reward(env: Env, total_stake_amount: i128) -> AnnualizedRewardResponse;

    fn query_withdrawable_reward(
        env: Env,
        address: Address,
        stakes: BondingInfo,
    ) -> WithdrawableRewardResponse;

    fn query_distributed_reward(env: Env, asset: Address) -> u128;

    fn query_undistributed_reward(env: Env, asset: Address) -> u128;
}

#[contractimpl]
impl StakingRewardsTrait for StakingRewards {
    #[allow(clippy::too_many_arguments)]
    fn initialize(
        env: Env,
        admin: Address,
        staking_contract: Address,
        reward_token: Address,
        max_complexity: u32,
        min_reward: i128,
        min_bond: i128,
    ) {
        if is_initialized(&env) {
            log!(
                &env,
                "Stake rewards: Initialize: initializing contract twice is not allowed"
            );
            panic_with_error!(&env, ContractError::AlreadyInitialized);
        }

        set_initialized(&env);

        env.events().publish(
            ("initialize", "StakingRewards rewards distribution contract"),
            (),
        );

        let config = Config {
            staking_contract,
            reward_token: reward_token.clone(),
            max_complexity,
            min_reward,
            min_bond,
        };
        save_config(&env, config);

        let distribution = Distribution {
            shares_per_point: 1u128,
            shares_leftover: 0u64,
            distributed_total: 0u128,
            withdrawable_total: 0u128,
            max_bonus_bps: 0u64,
            bonus_per_day_bps: 0u64,
        };

        save_distribution(&env, &reward_token, &distribution);
        // Create the default reward distribution curve which is just a flat 0 const
        save_reward_curve(&env, reward_token.clone(), &Curve::Constant(0));

        env.events()
            .publish(("create_distribution_flow", "asset"), &reward_token);

        utils::save_admin(&env, &admin);
    }

    fn add_user(env: Env, user: Address, stakes: BondingInfo) {
        let config = get_config(&env);
        // only Staking contract which deployed this one can call this method
        config.staking_contract.require_auth();
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        let new_power = calc_power(&config, stakes.total_stake, Decimal::one(), TOKEN_PER_POWER);
        let mut distribution = get_distribution(&env, &config.reward_token);
        update_rewards(
            &env,
            &user,
            &config.reward_token,
            &mut distribution,
            0, // old_rewards power is 0 when user didn't register before
            new_power,
        );

        env.events().publish(("stake_rewards", "add_user"), &user);
    }

    fn calculate_bond(env: Env, sender: Address, stakes: BondingInfo) {
        let config = get_config(&env);
        // only Staking contract which deployed this one can call this method
        config.staking_contract.require_auth();
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        let mut distribution = get_distribution(&env, &config.reward_token);
        let last_stake = stakes.stakes.last().unwrap();

        let old_power = calc_power(&config, stakes.total_stake, Decimal::one(), TOKEN_PER_POWER); // while bonding we use Decimal::one()
        let new_power = calc_power(
            &config,
            stakes.total_stake + last_stake.stake,
            Decimal::one(),
            TOKEN_PER_POWER,
        );
        update_rewards(
            &env,
            &sender,
            &config.reward_token,
            &mut distribution,
            old_power,
            new_power,
        );

        env.events().publish(("calculate_bond", "user"), &sender);
    }

    fn calculate_unbond(env: Env, sender: Address, stakes: BondingInfo, removed_stake: i128) {
        sender.require_auth();
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        let config = get_config(&env);
        // only Staking contract which deployed this one can call this method
        config.staking_contract.require_auth();

        // check for rewards and withdraw them
        let found_rewards: WithdrawableRewardResponse =
            Self::query_withdrawable_reward(env.clone(), sender.clone(), stakes.clone());

        if found_rewards.reward_amount != 0 {
            Self::withdraw_rewards(env.clone(), sender.clone(), stakes.clone());
        }

        let mut distribution = get_distribution(&env, &config.reward_token);

        let old_power = calc_power(&config, stakes.total_stake, Decimal::one(), TOKEN_PER_POWER); // while bonding we use Decimal::one()
        let new_power = calc_power(
            &config,
            stakes.total_stake - removed_stake,
            Decimal::one(),
            TOKEN_PER_POWER,
        );
        update_rewards(
            &env,
            &sender,
            &config.reward_token,
            &mut distribution,
            old_power,
            new_power,
        );

        env.events().publish(("calculate_unbond", "user"), &sender);
    }

    fn distribute_rewards(env: Env, total_staked_amount: i128) {
        let config = get_config(&env);
        // only Staking contract which deployed this one can call this method
        config.staking_contract.require_auth();
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        let calc_power_result = calc_power(
            &config,
            total_staked_amount,
            Decimal::one(),
            TOKEN_PER_POWER,
        );
        let total_rewards_power = convert_i128_to_u128(calc_power_result);

        if total_rewards_power == 0 {
            log!(&env, "Stake rewards: No rewards to distribute!");
            return;
        }
        let mut distribution = get_distribution(&env, &config.reward_token);
        let withdrawable = distribution.withdrawable_total;

        let reward_token_client = token_contract::Client::new(&env, &config.reward_token);
        // Undistributed rewards are simply all tokens left on the contract
        let undistributed_rewards_balance =
            reward_token_client.balance(&env.current_contract_address());
        let undistributed_rewards = convert_i128_to_u128(undistributed_rewards_balance);

        let curve = get_reward_curve(&env, &config.reward_token).expect("Stake: Distribute reward: Not reward curve exists, probably distribution haven't been created");

        // Calculate how much we have received since the last time Distributed was called,
        // including only the reward config amount that is eligible for distribution.
        // This is the amount we will distribute to all mem
        let amount = undistributed_rewards - withdrawable - curve.value(env.ledger().timestamp());

        if amount == 0 {
            return;
        }

        let leftover: u128 = distribution.shares_leftover.into();
        let points = (amount << SHARES_SHIFT) + leftover;
        let points_per_share = points / total_rewards_power;
        distribution.shares_leftover = (points % total_rewards_power) as u64;

        // Everything goes back to 128-bits/16-bytes
        // Full amount is added here to total withdrawable, as it should not be considered on its own
        // on future distributions - even if because of calculation offsets it is not fully
        // distributed, the error is handled by leftover.
        distribution.shares_per_point += points_per_share;
        distribution.distributed_total += amount;
        distribution.withdrawable_total += amount;

        save_distribution(&env, &config.reward_token, &distribution);

        env.events().publish(
            ("distribute_rewards", "asset"),
            &reward_token_client.address,
        );
        env.events()
            .publish(("distribute_rewards", "amount"), amount);
    }

    fn withdraw_rewards(env: Env, sender: Address, stakes: BondingInfo) {
        env.events().publish(("withdraw_rewards", "user"), &sender);
        let config = get_config(&env);
        // only Staking contract which deployed this one can call this method
        config.staking_contract.require_auth();
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        // get distribution data for the given reward
        let mut distribution = get_distribution(&env, &config.reward_token);
        // get withdraw adjustment for the given distribution
        let mut withdraw_adjustment = get_withdraw_adjustment(&env, &sender, &config.reward_token);
        // calculate current reward amount given the distribution and subtracting withdraw
        // adjustments
        let reward_amount = withdrawable_rewards(
            stakes.total_stake,
            &distribution,
            &withdraw_adjustment,
            &config,
        );

        // calculate the actual reward amounts - each stake is worth 1/60th per each staked day
        let reward_multiplier = calc_withdraw_power(&env, &stakes.stakes);
        let reward_amount = convert_u128_to_i128(reward_amount) * reward_multiplier;

        if reward_amount == 0 {
            return;
        }
        withdraw_adjustment.withdrawn_rewards += reward_amount as u128;
        distribution.withdrawable_total -= reward_amount as u128;

        save_distribution(&env, &config.reward_token, &distribution);
        save_withdraw_adjustment(&env, &sender, &config.reward_token, &withdraw_adjustment);

        let reward_token_client = token_contract::Client::new(&env, &config.reward_token);
        reward_token_client.transfer(&env.current_contract_address(), &sender, &reward_amount);

        env.events().publish(
            ("withdraw_rewards", "reward_token"),
            &reward_token_client.address,
        );
        env.events()
            .publish(("withdraw_rewards", "reward_amount"), reward_amount);
    }

    fn fund_distribution(
        env: Env,
        start_time: u64,
        distribution_duration: u64,
        token_amount: i128,
    ) {
        let admin = get_admin(&env);
        admin.require_auth();
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        let config = get_config(&env);
        // only Staking contract which deployed this one can call this method
        config.staking_contract.require_auth();

        // Load previous reward curve; it must exist if the distribution exists
        // In case of first time funding, it will be a constant 0 curve
        let previous_reward_curve = get_reward_curve(&env, &config.reward_token).expect("Stake rewards: Fund distribution: Not reward curve exists, probably distribution haven't been created");
        let max_complexity = config.max_complexity;

        let current_time = env.ledger().timestamp();
        if start_time < current_time {
            log!(
                &env,
                "Stake rewards: Fund distribution: Fund distribution start time is too early"
            );
            panic_with_error!(&env, ContractError::InvalidTime);
        }

        if config.min_reward > token_amount {
            log!(
                &env,
                "Stake rewards: Fund distribution: minimum reward amount not reached",
            );
            panic_with_error!(&env, ContractError::MinRewardNotEnough);
        }

        // transfer tokens to fund distribution
        let reward_token_client = token_contract::Client::new(&env, &config.reward_token);
        reward_token_client.transfer(&admin, &env.current_contract_address(), &token_amount);

        let end_time = current_time + distribution_duration;
        // define a distribution curve starting at start_time with token_amount of tokens
        // and ending at end_time with 0 tokens
        let new_reward_distribution = Curve::saturating_linear(
            (start_time, convert_i128_to_u128(token_amount)),
            (end_time, 0),
        );

        // Validate the the curve locks at most the amount provided and
        // also fully unlocks all rewards sent
        let (min, max) = new_reward_distribution.range();
        if min != 0 || max > convert_i128_to_u128(token_amount) {
            log!(
                &env,
                "Stake rewards: Fund distribution: Rewards validation failed"
            );
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
                            "Stake rewards: Fund distribution: Curve complexity validation failed"
                        );
                        panic_with_error!(&env, ContractError::InvalidMaxComplexity);
                    });
            }
        }

        save_reward_curve(&env, config.reward_token.clone(), &new_reward_curve);

        env.events()
            .publish(("fund_reward_distribution", "asset"), &config.reward_token);
        env.events()
            .publish(("fund_reward_distribution", "amount"), token_amount);
        env.events()
            .publish(("fund_reward_distribution", "start_time"), start_time);
        env.events()
            .publish(("fund_reward_distribution", "end_time"), end_time);
    }

    // QUERIES

    fn query_config(env: Env) -> ConfigResponse {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        ConfigResponse {
            config: get_config(&env),
        }
    }

    fn query_admin(env: Env) -> Address {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        get_admin(&env)
    }

    fn query_annualized_reward(env: Env, total_staked_amount: i128) -> AnnualizedRewardResponse {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        let now = env.ledger().timestamp();
        let config = get_config(&env);

        let total_stake_power = calc_power(
            &config,
            total_staked_amount,
            Decimal::one(),
            TOKEN_PER_POWER,
        );
        if total_stake_power == 0 {
            return AnnualizedRewardResponse {
                asset: config.reward_token.clone(),
                amount: String::from_str(&env, "0"),
            };
        }

        // get distribution data for the given reward
        let distribution = get_distribution(&env, &config.reward_token);
        let curve = get_reward_curve(&env, &config.reward_token);
        let annualized_payout = calculate_annualized_payout(curve, now);
        let apr = annualized_payout
            / convert_u128_to_i128(
                convert_i128_to_u128(total_stake_power) * distribution.shares_per_point,
            );

        AnnualizedRewardResponse {
            asset: config.reward_token.clone(),
            amount: apr.to_string(&env),
        }
    }

    fn query_withdrawable_reward(
        env: Env,
        user: Address,
        stakes: BondingInfo,
    ) -> WithdrawableRewardResponse {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        let config = get_config(&env);
        // iterate over all distributions and calculate withdrawable rewards
        // get distribution data for the given reward
        let distribution = get_distribution(&env, &config.reward_token);
        // get withdraw adjustment for the given distribution
        let withdraw_adjustment = get_withdraw_adjustment(&env, &user, &config.reward_token);
        // calculate current reward amount given the distribution and subtracting withdraw
        // adjustments
        let reward_amount = withdrawable_rewards(
            stakes.total_stake,
            &distribution,
            &withdraw_adjustment,
            &config,
        );

        // calculate the actual reward amounts - each stake is worth 1/60th per each staked day
        let reward_multiplier = calc_withdraw_power(&env, &stakes.stakes);

        let reward_amount =
            convert_i128_to_u128(convert_u128_to_i128(reward_amount) * reward_multiplier);

        WithdrawableRewardResponse {
            reward_address: config.reward_token,
            reward_amount,
        }
    }

    fn query_distributed_reward(env: Env, asset: Address) -> u128 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        let distribution = get_distribution(&env, &asset);
        distribution.distributed_total
    }

    fn query_undistributed_reward(env: Env, asset: Address) -> u128 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        let distribution = get_distribution(&env, &asset);
        let reward_token_client = token_contract::Client::new(&env, &asset);
        let reward_token_balance = reward_token_client.balance(&env.current_contract_address());
        convert_i128_to_u128(reward_token_balance) - distribution.withdrawable_total
    }
}

#[contractimpl]
impl StakingRewards {
    #[allow(dead_code)]
    pub fn update(env: Env, new_wasm_hash: BytesN<32>) {
        let admin = get_admin(&env);
        admin.require_auth();

        env.deployer().update_current_contract_wasm(new_wasm_hash);
    }
}
