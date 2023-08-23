use soroban_sdk::{contract, contractimpl, contractmeta, log, vec, Address, Env, Vec};

use crate::{
    distribution::{
        get_distribution, get_reward_curve, get_withdraw_adjustment, save_distribution,
        save_reward_curve, save_withdraw_adjustment, update_rewards, withdrawable_rewards,
        Distribution, SHARES_SHIFT,
    },
    error::ContractError,
    msg::{
        AnnualizedRewardsResponse, ConfigResponse, StakedResponse, WithdrawableReward,
        WithdrawableRewardsResponse,
    },
    storage::{
        get_config, get_stakes, save_config, save_stakes, update_stakes_rewards,
        utils::{
            self, add_distribution, get_admin, get_distributions, get_total_staked_counter,
            get_total_virtual_staked_counter, reset_stake_increase_counter,
        },
        Config, Stake,
    },
    token_contract,
};
use curve::Curve;
use decimal::Decimal;

// Metadata that is added on to the WASM custom section
contractmeta!(
    key = "Description",
    val = "Phoenix Protocol LP Share token staking"
);

#[contract]
pub struct Staking;

pub trait StakingTrait {
    // Sets the token contract addresses for this pool
    // epoch: Number of seconds between payments
    #[allow(clippy::too_many_arguments)]
    fn initialize(
        env: Env,
        admin: Address,
        lp_token: Address,
        min_bond: i128,
        max_distributions: u32,
        min_reward: i128,
        bonus_per_day_bps: i64,
        max_bonus_bps: i64,
    ) -> Result<(), ContractError>;

    fn bond(env: Env, sender: Address, tokens: i128) -> Result<(), ContractError>;

    fn unbond(
        env: Env,
        sender: Address,
        stake_amount: i128,
        stake_timestamp: u64,
    ) -> Result<(), ContractError>;

    fn create_distribution_flow(
        env: Env,
        sender: Address,
        manager: Address,
        asset: Address,
    ) -> Result<(), ContractError>;

    fn distribute_rewards(env: Env) -> Result<(), ContractError>;

    fn withdraw_rewards(env: Env, sender: Address) -> Result<(), ContractError>;

    fn fund_distribution(
        env: Env,
        sender: Address,
        start_time: u64,
        distribution_duration: u64,
        token_address: Address,
        token_amount: i128,
    ) -> Result<(), ContractError>;

    // QUERIES

    fn query_config(env: Env) -> Result<ConfigResponse, ContractError>;

    fn query_admin(env: Env) -> Result<Address, ContractError>;

    fn query_staked(env: Env, address: Address) -> Result<StakedResponse, ContractError>;

    fn query_total_staked(env: Env) -> Result<i128, ContractError>;

    fn query_annualized_rewards(env: Env) -> Result<AnnualizedRewardsResponse, ContractError>;

    fn query_withdrawable_rewards(
        env: Env,
        address: Address,
    ) -> Result<WithdrawableRewardsResponse, ContractError>;

    fn query_distributed_rewards(env: Env, asset: Address) -> Result<u128, ContractError>;

    fn query_undistributed_rewards(env: Env, asset: Address) -> Result<u128, ContractError>;
}

#[contractimpl]
impl StakingTrait for Staking {
    #[allow(clippy::too_many_arguments)]
    fn initialize(
        env: Env,
        admin: Address,
        lp_token: Address,
        min_bond: i128,
        max_distributions: u32,
        min_reward: i128,
        bonus_per_day_bps: i64,
        max_bonus_bps: i64,
    ) -> Result<(), ContractError> {
        if min_bond <= 0 {
            log!(
                &env,
                "Minimum amount of lp share tokens to bond can not be smaller or equal to 0"
            );
            return Err(ContractError::MinStakeLessOrEqualZero);
        }
        if min_reward <= 0 {
            log!(&env, "min_reward must be bigger then 0!");
            return Err(ContractError::MinRewardTooSmall);
        }

        env.events()
            .publish(("initialize", "LP Share token staking contract"), &lp_token);

        let config = Config {
            lp_token,
            min_bond,
            max_distributions,
            min_reward,
            bonus_per_day_bps,
            max_bonus_bps,
        };
        save_config(&env, config);

        utils::save_admin(&env, &admin);
        utils::init_total_staked(&env);

        Ok(())
    }

    fn bond(env: Env, sender: Address, tokens: i128) -> Result<(), ContractError> {
        sender.require_auth();

        let ledger = env.ledger();
        let config = get_config(&env)?;

        if tokens < config.min_bond {
            log!(
                &env,
                "Trying to bond {} which is less then minimum {} required!",
                tokens,
                config.min_bond
            );
            return Err(ContractError::StakeLessThenMinBond);
        }

        let lp_token_client = token_contract::Client::new(&env, &config.lp_token);
        lp_token_client.transfer(&sender, &env.current_contract_address(), &tokens);

        let mut stakes = get_stakes(&env, &sender)?;
        let stake = Stake {
            stake: tokens,
            stake_timestamp: ledger.timestamp(),
        };
        stakes.total_stake += tokens as u128;
        stakes.virtual_stake += tokens as u128;
        // TODO: Discuss: Add implementation to add stake if another is present in +-24h timestamp to avoid
        // creating multiple stakes the same day

        let total_staked = utils::get_total_staked_counter(&env)?;
        for distribution_address in get_distributions(&env) {
            let mut distribution = get_distribution(&env, &distribution_address)?;
            update_rewards(
                &env,
                &sender,
                &distribution_address,
                &mut distribution,
                total_staked,
                total_staked + tokens,
            )
        }

        stakes.stakes.push_back(stake);
        save_stakes(&env, &sender, &stakes);
        utils::increase_total_staked(&env, &tokens)?;
        utils::increase_total_virtual_staked(&env, &tokens)?;

        env.events().publish(("bond", "user"), &sender);
        env.events().publish(("bond", "token"), &config.lp_token);
        env.events().publish(("bond", "amount"), tokens);

        Ok(())
    }

    fn unbond(
        env: Env,
        sender: Address,
        stake_amount: i128,
        stake_timestamp: u64,
    ) -> Result<(), ContractError> {
        sender.require_auth();

        let config = get_config(&env)?;

        let mut stakes = get_stakes(&env, &sender)?;
        remove_stake(&mut stakes.stakes, stake_amount, stake_timestamp)?;
        stakes.total_stake -= stake_amount as u128;

        // calculate new virtual stake amount necessary for correct reward calculation
        let virtual_stake_difference = stakes.virtual_stake - stakes.total_stake;
        // withdrawing any stake will reset the virtual stake to the current stake amount,
        // effectively discarding the bonus
        stakes.virtual_stake = stakes.total_stake;
        stakes.current_rewards_bps = 0i64;

        let lp_token_client = token_contract::Client::new(&env, &config.lp_token);
        lp_token_client.transfer(&env.current_contract_address(), &sender, &stake_amount);

        save_stakes(&env, &sender, &stakes);
        utils::decrease_total_staked(&env, &stake_amount)?;
        utils::decrease_total_virtual_staked(&env, &(virtual_stake_difference as i128))?;

        env.events().publish(("unbond", "user"), &sender);
        env.events().publish(("bond", "token"), &config.lp_token);
        env.events().publish(("bond", "amount"), stake_amount);

        Ok(())
    }

    fn create_distribution_flow(
        env: Env,
        sender: Address,
        manager: Address,
        asset: Address,
    ) -> Result<(), ContractError> {
        sender.require_auth();

        let distribution = Distribution {
            shares_per_point: 1u128,
            shares_leftover: 0u64,
            distributed_total: 0u128,
            withdrawable_total: 0u128,
            manager,
        };

        let reward_token_client = token_contract::Client::new(&env, &asset);
        // add distribution to the vector of distributions
        add_distribution(&env, &reward_token_client.address)?;
        save_distribution(&env, &reward_token_client.address, &distribution);
        // Create the default reward distribution curve which is just a flat 0 const
        save_reward_curve(&env, asset, &Curve::Constant(0));

        env.events().publish(
            ("create_distribution_flow", "asset"),
            &reward_token_client.address,
        );

        Ok(())
    }

    fn distribute_rewards(env: Env) -> Result<(), ContractError> {
        // get total_virtual_staked, meaning all staked tokens plus increases coming from rewards
        let total_rewards_power = get_total_virtual_staked_counter(&env)? as u128;
        if total_rewards_power == 0 {
            log!(&env, "No rewards to distribute!");
            return Ok(());
        }

        let config = get_config(&env)?;

        for distribution_address in get_distributions(&env) {
            let mut distribution = get_distribution(&env, &distribution_address)?;
            let withdrawable = distribution.withdrawable_total;

            let reward_token_client = token_contract::Client::new(&env, &distribution_address);
            // Undistributed rewards are simply all tokens left on the contract
            let undistributed_rewards =
                reward_token_client.balance(&env.current_contract_address()) as u128;

            let curve = get_reward_curve(&env, &distribution_address)?;

            // Calculate how much we have received since the last time Distributed was called,
            // including only the reward config amount that is eligible for distribution.
            // This is the amount we will distribute to all mem
            let amount =
                undistributed_rewards - withdrawable - curve.value(env.ledger().timestamp());

            if amount == 0 {
                continue;
            }

            let leftover: u128 = distribution.shares_leftover.into();
            let points = (amount << SHARES_SHIFT) + leftover;

            // One line that separates my life from being an unfeeling ghost after spending days on
            // debugging the issue - why the stake rewards increased after increasing user stakes;
            // the answer is - total number of points needs to be increased by percentage of global
            // stake increase caused by rewards withdrawals
            let stake_increase = Decimal::one()
                - Decimal::bps(config.bonus_per_day_bps)
                    * Decimal::from_ratio(utils::get_stake_increase_counter(&env), 1);

            let points = (points as i128 * stake_increase) as u128;
            let points_per_share = points / total_rewards_power;
            distribution.shares_leftover = (points % total_rewards_power) as u64;

            // Everything goes back to 128-bits/16-bytes
            // Full amount is added here to total withdrawable, as it should not be considered on its own
            // on future distributions - even if because of calculation offsets it is not fully
            // distributed, the error is handled by leftover.
            distribution.shares_per_point += points_per_share;
            distribution.distributed_total += amount;
            distribution.withdrawable_total += amount;

            save_distribution(&env, &distribution_address, &distribution);

            env.events().publish(
                ("distribute_rewards", "asset"),
                &reward_token_client.address,
            );
            env.events()
                .publish(("distribute_rewards", "amount"), amount);
        }

        // reset counter of total stake increase due to bonus rewards added during withdrawals
        reset_stake_increase_counter(&env);

        Ok(())
    }

    fn withdraw_rewards(env: Env, sender: Address) -> Result<(), ContractError> {
        env.events().publish(("withdraw_rewards", "user"), &sender);

        for distribution_address in get_distributions(&env) {
            // get distribution data for the given reward
            let mut distribution = get_distribution(&env, &distribution_address)?;
            // get withdraw adjustment for the given distribution
            let mut withdraw_adjustment =
                get_withdraw_adjustment(&env, &sender, &distribution_address);
            // calculate current reward amount given the distribution and subtracting withdraw
            // adjustments
            let reward_amount =
                withdrawable_rewards(&env, &sender, &distribution, &withdraw_adjustment)?;

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
                .publish(("withdraw_rewards", "reward_amount"), reward_amount as i128);
        }

        save_withdraw_adjustments(&env, &sender, &updated_withdraw_adjustments);
        update_stakes_rewards(&env, &sender)?;

        Ok(())
    }

    fn fund_distribution(
        env: Env,
        sender: Address,
        start_time: u64,
        distribution_duration: u64,
        token_address: Address,
        token_amount: i128,
    ) -> Result<(), ContractError> {
        sender.require_auth();

        // Load previous reward curve; it must exist if the distribution exists
        // In case of first time funding, it will be a constant 0 curve
        let previous_reward_curve = get_reward_curve(&env, &token_address)?;

        let current_time = env.ledger().timestamp();
        if start_time < current_time {
            log!(
                &env,
                "Trying to fund distribution flow with start timestamp: {} which is earlier then the current one: {}",
                start_time,
                current_time
            );
            return Err(ContractError::FundDistributionStartTimeTooEarly);
        }

        let config = get_config(&env)?;
        if config.min_reward > token_amount {
            log!(
                &env,
                "Trying to create distribution flow with reward not reaching minimum amount: {}",
                config.min_reward
            );
            return Err(ContractError::MinRewardNotReached);
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
            log!(
                &env,
                "Trying to create reward distribution which either doesn't end with empty balance or exceeds provided amount"
            );
            return Err(ContractError::RewardsValidationFailed);
        }

        // now combine old distribution with the new schedule
        let new_reward_curve = previous_reward_curve.combine(&env, &new_reward_distribution);
        save_reward_curve(&env, token_address.clone(), &new_reward_curve);

        env.events()
            .publish(("fund_reward_distribution", "asset"), &token_address);
        env.events()
            .publish(("fund_reward_distribution", "amount"), token_amount);
        env.events()
            .publish(("fund_reward_distribution", "start_time"), start_time);
        env.events()
            .publish(("fund_reward_distribution", "end_time"), end_time);

        Ok(())
    }

    // QUERIES

    fn query_config(env: Env) -> Result<ConfigResponse, ContractError> {
        Ok(ConfigResponse {
            config: get_config(&env)?,
        })
    }

    fn query_admin(env: Env) -> Result<Address, ContractError> {
        get_admin(&env)
    }

    fn query_staked(env: Env, address: Address) -> Result<StakedResponse, ContractError> {
        Ok(StakedResponse {
            stakes: get_stakes(&env, &address)?.stakes,
        })
    }

    fn query_total_staked(env: Env) -> Result<i128, ContractError> {
        get_total_staked_counter(&env)
    }

    fn query_annualized_rewards(_env: Env) -> Result<AnnualizedRewardsResponse, ContractError> {
        unimplemented!();
    }

    fn query_withdrawable_rewards(
        env: Env,
        user: Address,
    ) -> Result<WithdrawableRewardsResponse, ContractError> {
        // iterate over all distributions and calculate withdrawable rewards
        let mut rewards = vec![&env];
        for distribution_address in get_distributions(&env) {
            // get distribution data for the given reward
            let distribution = get_distribution(&env, &distribution_address)?;
            // get withdraw adjustment for the given distribution
            let withdraw_adjustment = get_withdraw_adjustment(&env, &user, &distribution_address);
            // calculate current reward amount given the distribution and subtracting withdraw
            // adjustments
            let reward_amount =
                withdrawable_rewards(&env, &user, &distribution, &withdraw_adjustment)?;
            rewards.push_back(WithdrawableReward {
                reward_address: distribution_address,
                reward_amount,
            });
        }

        Ok(WithdrawableRewardsResponse { rewards })
    }

    fn query_distributed_rewards(env: Env, asset: Address) -> Result<u128, ContractError> {
        let distribution = get_distribution(&env, &asset)?;
        Ok(distribution.distributed_total)
    }

    fn query_undistributed_rewards(env: Env, asset: Address) -> Result<u128, ContractError> {
        let distribution = get_distribution(&env, &asset)?;
        let reward_token_client = token_contract::Client::new(&env, &asset);
        let undistributed_rewards = reward_token_client.balance(&env.current_contract_address())
            as u128
            - distribution.withdrawable_total;
        Ok(undistributed_rewards)
    }
}

// Function to remove a stake from the vector
fn remove_stake(
    stakes: &mut Vec<Stake>,
    stake: i128,
    stake_timestamp: u64,
) -> Result<(), ContractError> {
    // Find the index of the stake that matches the given stake and stake_timestamp
    if let Some(index) = stakes
        .iter()
        .position(|s| s.stake == stake && s.stake_timestamp == stake_timestamp)
    {
        // Remove the stake at the found index
        stakes.remove(index as u32);
        Ok(())
    } else {
        // Stake not found, return an error
        Err(ContractError::StakeNotFound)
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
        let result = remove_stake(&mut stakes, stake_to_remove, stake_timestamp_to_remove);
        assert!(result.is_ok());

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
    fn test_remove_stake_not_found() {
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

        // Check that the stake is not found and returns an error
        let result = remove_stake(&mut stakes, 100, 2);
        assert!(result.is_err());
        let result = remove_stake(&mut stakes, 200, 1);
        assert!(result.is_err());
        let result = remove_stake(&mut stakes, 150, 1);
        assert!(result.is_err());

        // Check that the vector remains unchanged
        assert_eq!(
            stakes,
            vec![
                &env,
                Stake {
                    stake: 100,
                    stake_timestamp: 1
                },
                Stake {
                    stake: 200,
                    stake_timestamp: 2
                },
                Stake {
                    stake: 150,
                    stake_timestamp: 3
                },
            ]
        );
    }
}
