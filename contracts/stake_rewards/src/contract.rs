use soroban_decimal::Decimal;
use soroban_sdk::{
    contract, contractimpl, contractmeta, log, panic_with_error, vec, Address, BytesN, Env, String,
    Vec,
};

use crate::distribution::calc_power;
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
    stake_contract,
    storage::{
        get_config, get_stakes, save_config, save_stakes,
        utils::{
            self, add_distribution, get_admin, get_distributions, get_total_staked_counter,
            is_initialized, set_initialized,
        },
        Config, Stake,
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

pub trait StakingRewardsTrait {
    // Sets the token contract addresses for this pool
    #[allow(clippy::too_many_arguments)]
    fn initialize(
        env: Env,
        admin: Address,
        staking_contract: Address,
        reward_token: Address,
        owner: Address,
    );

    fn bond(env: Env, sender: Address, tokens: i128);

    fn unbond(env: Env, sender: Address, stake_amount: i128, stake_timestamp: u64);

    fn distribute_rewards(env: Env);

    fn withdraw_rewards(env: Env, sender: Address);

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

    fn query_distributed_rewards(env: Env, asset: Address) -> u128;

    fn query_undistributed_rewards(env: Env, asset: Address) -> u128;
}

#[contractimpl]
impl StakingRewardsTrait for StakingRewards {
    #[allow(clippy::too_many_arguments)]
    fn initialize(
        env: Env,
        admin: Address,
        staking_contract: Address,
        reward_token: Address,
        owner: Address,
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
            &(),
        );

        let config = Config {
            staking_contract,
            owner,
            reward_token,
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

        // add distribution to the vector of distributions
        add_distribution(&env, &reward_token);
        save_distribution(&env, &reward_token, &distribution);
        // Create the default reward distribution curve which is just a flat 0 const
        save_reward_curve(&env, asset, &Curve::Constant(0));

        env.events().publish(
            ("create_distribution_flow", "asset"),
            &reward_token_client.address,
        );

        utils::save_admin(&env, &admin);
        utils::init_total_staked(&env);
    }

    fn calculate_bond(env: Env, sender: Address) {
        sender.require_auth();

        let ledger = env.ledger();
        let config = get_config(&env);

        let stake_client = stake_contract::Client::new(&env, &config.lp_token);
        let stakes = stake_client.query_staked(&sender);

        let now = ledger.timestamp();

        let mut distribution = get_distribution(&env, &config.reward_token);

        let old_power = calc_power(&config, stakes.total_stake, Decimal::one(), TOKEN_PER_POWER); // while bonding we use Decimal::one()
        let new_power = calc_power(
            &config,
            stakes.total_stake + tokens,
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

    fn calculate_unbond(env: Env, sender: Address, stake_amount: i128, stake_timestamp: u64) {
        sender.require_auth();

        let config = get_config(&env);

        // check for rewards and withdraw them
        let found_rewards: WithdrawableRewardsResponse =
            Self::query_withdrawable_rewards(env.clone(), sender.clone());

        if !found_rewards.rewards.is_empty() {
            Self::withdraw_rewards(env.clone(), sender.clone());
        }

        let mut distribution = get_distribution(&env, &config.reward_token);

        let stake_client = stake_contract::Client::new(&env, &config.lp_token);
        let stakes = stake_client.query_staked(&sender);

        // TODO FIXME: This is wrong, because the last stake would be removed already
        // maybe call calculate_unbond first?
        let mut last_stake = stakes.stakes.last().unwrap_or_default();

        let old_power = calc_power(&config, stakes.total_stake, Decimal::one(), TOKEN_PER_POWER); // while bonding we use Decimal::one()
        let new_power = calc_power(
            &config,
            stakes.total_stake - last_stake.stake,
            Decimal::one(),
            TOKEN_PER_POWER,
        );
        update_rewards(
            &env,
            &sender,
            &distribution_address,
            &mut distribution,
            old_power,
            new_power,
        );

        env.events().publish(("calculate_unbond", "user"), &sender);
        env.events().publish(("bond", "token"), &config.lp_token);
        env.events().publish(("bond", "amount"), stake_amount);
    }

    fn distribute_rewards(env: Env) {
        let config = get_config(&env);

        let stake_client = stake_contract::Client::new(&env, &config.lp_token);
        let total_staked_amount = stake_client.query_total_staked(&sender);
        let total_rewards_power = calc_power(
            &config,
            total_staked_amount,
            Decimal::one(),
            TOKEN_PER_POWER,
        ) as u128;

        if total_rewards_power == 0 {
            log!(&env, "Stake rewards: No rewards to distribute!");
            return;
        }
        let mut distribution = get_distribution(&env, &config.reward_token);
        let withdrawable = distribution.withdrawable_total;

        let reward_token_client = token_contract::Client::new(&env, &distribution_address);
        // Undistributed rewards are simply all tokens left on the contract
        let undistributed_rewards =
            reward_token_client.balance(&env.current_contract_address()) as u128;

        let curve = get_reward_curve(&env, &config.reward_token).expect("Stake: Distribute reward: Not reward curve exists, probably distribution haven't been created");

        // Calculate how much we have received since the last time Distributed was called,
        // including only the reward config amount that is eligible for distribution.
        // This is the amount we will distribute to all mem
        let amount = undistributed_rewards - withdrawable - curve.value(env.ledger().timestamp());

        if amount == 0 {
            continue;
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

    fn withdraw_rewards(env: Env, sender: Address) {
        env.events().publish(("withdraw_rewards", "user"), &sender);
        let config = get_config(&env);

        // get distribution data for the given reward
        let mut distribution = get_distribution(&env, &config.reward_token);
        // get withdraw adjustment for the given distribution
        let mut withdraw_adjustment = get_withdraw_adjustment(&env, &sender, &distribution_address);
        // calculate current reward amount given the distribution and subtracting withdraw
        // adjustments
        let reward_amount =
            withdrawable_rewards(&env, &sender, &distribution, &withdraw_adjustment, &config);

        if reward_amount == 0 {
            continue;
        }
        withdraw_adjustment.withdrawn_rewards += reward_amount;
        distribution.withdrawable_total -= reward_amount;

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

    fn fund_distribution(
        env: Env,
        sender: Address,
        start_time: u64,
        distribution_duration: u64,
        token_address: Address,
        token_amount: i128,
    ) {
        sender.require_auth();

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

        let end_time = current_time + distribution_duration;
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
            .publish(("fund_reward_distribution", "asset"), &token_address);
        env.events()
            .publish(("fund_reward_distribution", "amount"), token_amount);
        env.events()
            .publish(("fund_reward_distribution", "start_time"), start_time);
        env.events()
            .publish(("fund_reward_distribution", "end_time"), end_time);
    }

    // QUERIES

    fn query_config(env: Env) -> ConfigResponse {
        ConfigResponse {
            config: get_config(&env),
        }
    }

    fn query_admin(env: Env) -> Address {
        get_admin(&env)
    }

    fn query_staked(env: Env, address: Address) -> StakedResponse {
        StakedResponse {
            stakes: get_stakes(&env, &address).stakes,
        }
    }

    fn query_total_staked(env: Env) -> i128 {
        get_total_staked_counter(&env)
    }

    fn query_annualized_rewards(env: Env) -> AnnualizedRewardsResponse {
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

    fn query_distributed_rewards(env: Env, asset: Address) -> u128 {
        let distribution = get_distribution(&env, &asset);
        distribution.distributed_total
    }

    fn query_undistributed_rewards(env: Env, asset: Address) -> u128 {
        let distribution = get_distribution(&env, &asset);
        let reward_token_client = token_contract::Client::new(&env, &asset);
        reward_token_client.balance(&env.current_contract_address()) as u128
            - distribution.withdrawable_total
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