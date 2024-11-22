use phoenix::ttl::{INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD};
use soroban_sdk::{
    contract, contractimpl, contractmeta, log, map, panic_with_error, vec, Address, BytesN, Env,
    Vec,
};

use crate::{
    distribution::{
        calculate_pending_rewards, get_reward_history, get_total_staked_history,
        save_reward_history, save_total_staked_history,
    },
    error::ContractError,
    msg::{ConfigResponse, StakedResponse, WithdrawableReward, WithdrawableRewardsResponse},
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

// Metadata that is added on to the WASM custom section
contractmeta!(
    key = "Description",
    val = "Phoenix Protocol LP Share token staking"
);

#[contract]
pub struct Staking;

#[allow(dead_code)]
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

    fn create_distribution_flow(env: Env, sender: Address, asset: Address);

    fn distribute_rewards(env: Env, sender: Address, amount: i128, reward_token: Address);

    fn withdraw_rewards(env: Env, sender: Address);

    // QUERIES

    fn query_config(env: Env) -> ConfigResponse;

    fn query_admin(env: Env) -> Address;

    fn query_staked(env: Env, address: Address) -> StakedResponse;

    fn query_total_staked(env: Env) -> i128;

    // fn query_annualized_rewards(env: Env) -> AnnualizedRewardsResponse;

    fn query_withdrawable_rewards(env: Env, address: Address) -> WithdrawableRewardsResponse;

    // fn query_distributed_rewards(env: Env, asset: Address) -> u128;

    // fn query_undistributed_rewards(env: Env, asset: Address) -> u128;
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

        utils::save_admin(&env, &admin);
        utils::init_total_staked(&env);
        save_total_staked_history(&env, map![&env]);
    }

    fn bond(env: Env, sender: Address, tokens: i128) {
        sender.require_auth();
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

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

        stakes.total_stake += tokens;
        let stake = Stake {
            stake: tokens,
            stake_timestamp: ledger.timestamp(),
        };
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
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        let config = get_config(&env);

        let mut stakes = get_stakes(&env, &sender);

        remove_stake(&env, &mut stakes.stakes, stake_amount, stake_timestamp);
        stakes.total_stake -= stake_amount;

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
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        let config = get_config(&env);
        if sender != config.manager && sender != config.owner {
            log!(env, "Stake: create distribution: Non-authorized creation!");
            panic_with_error!(&env, ContractError::Unauthorized);
        }

        add_distribution(&env, &asset);
        save_reward_history(&env, &asset, map![&env]);

        env.events()
            .publish(("create_distribution_flow", "asset"), &asset);
    }

    fn distribute_rewards(env: Env, sender: Address, amount: i128, reward_token: Address) {
        sender.require_auth();

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        let config = get_config(&env);
        if sender != config.manager && sender != config.owner {
            log!(env, "Stake: create distribution: Non-authorized creation!");
            panic_with_error!(&env, ContractError::Unauthorized);
        }

        if !get_distributions(&env).contains(&reward_token) {
            log!(
                env,
                "Stake: Distribute rewards: No distribution for this reward token exists!"
            );
            panic_with_error!(&env, ContractError::DistributionNotFound);
        }

        let current_timestamp = env.ledger().timestamp();
        let total_staked_amount = get_total_staked_counter(&env);

        let mut total_staked_history = get_total_staked_history(&env);
        total_staked_history.set(current_timestamp, total_staked_amount as u128);
        save_total_staked_history(&env, total_staked_history);

        let mut reward_history = get_reward_history(&env, &reward_token);
        reward_history.set(current_timestamp, amount as u128);
        save_reward_history(&env, &reward_token, reward_history);

        token_contract::Client::new(&env, &reward_token).transfer(
            &sender,
            &env.current_contract_address(),
            &amount,
        );

        env.events()
            .publish(("distribute_rewards", "asset"), &reward_token);
    }

    fn withdraw_rewards(env: Env, sender: Address) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        env.events().publish(("withdraw_rewards", "user"), &sender);

        let mut stakes = get_stakes(&env, &sender);

        for asset in get_distributions(&env).iter() {
            let pending_reward = calculate_pending_rewards(&env, &asset, &stakes);
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

    fn query_staked(env: Env, address: Address) -> StakedResponse {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        let stakes = get_stakes(&env, &address);
        StakedResponse {
            stakes: stakes.stakes,
            total_stake: stakes.total_stake,
            last_reward_time: stakes.last_reward_time,
        }
    }

    fn query_total_staked(env: Env) -> i128 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        get_total_staked_counter(&env)
    }

    // fn query_annualized_rewards(env: Env) -> AnnualizedRewardsResponse {
    //     let mut aprs = vec![&env];
    //     let total_stake_amount = get_total_staked_counter(&env);
    //     let apr_fn_arg: Val = total_stake_amount.into_val(&env);

    //     for asset in get_distributions(&env) {
    //         let apr: AnnualizedReward = env.invoke_contract(
    //             &distribution_address,
    //             &Symbol::new(&env, "query_annualized_reward"),
    //             vec![&env, apr_fn_arg],
    //         );

    //         aprs.push_back(AnnualizedReward {
    //             asset,
    //             amount: apr.amount,
    //         });
    //     }

    //     AnnualizedRewardsResponse { rewards: aprs }
    // }

    fn query_withdrawable_rewards(env: Env, user: Address) -> WithdrawableRewardsResponse {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        let stakes = get_stakes(&env, &user);
        // iterate over all distributions and calculate withdrawable rewards
        let mut rewards = vec![&env];
        for asset in get_distributions(&env) {
            let pending_reward = calculate_pending_rewards(&env, &asset, &stakes);

            rewards.push_back(WithdrawableReward {
                reward_address: asset,
                reward_amount: pending_reward as u128,
            });
        }

        WithdrawableRewardsResponse { rewards }
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

    // fn query_undistributed_rewards(env: Env, asset: Address) -> u128 {
    //     let staking_rewards = find_stake_rewards_by_asset(&env, &asset).unwrap();
    //     let unds_rew_fn_arg: Val = asset.into_val(&env);
    //     let ret: u128 = env.invoke_contract(
    //         &staking_rewards,
    //         &Symbol::new(&env, "query_undistributed_reward"),
    //         vec![&env, unds_rew_fn_arg],
    //     );
    //     ret
    // }
}

#[contractimpl]
impl Staking {
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
