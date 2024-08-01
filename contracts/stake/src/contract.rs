use soroban_sdk::{
    contract, contractimpl, contractmeta, log, panic_with_error, vec, Address, BytesN, Env,
    IntoVal, Symbol, Val, Vec,
};

use crate::{
    distribution::get_distribution,
    error::ContractError,
    msg::{ConfigResponse, StakedResponse, WithdrawableRewardResponse},
    storage::{
        get_config, get_stakes, save_config, save_stakes,
        utils::{
            self, get_admin, get_stake_rewards, get_total_staked_counter, is_initialized,
            set_initialized, set_stake_rewards,
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
        stake_rewards: Address,
        min_bond: i128,
        min_reward: i128,
        manager: Address,
        owner: Address,
        max_complexity: u32,
    );

    fn bond(env: Env, sender: Address, tokens: i128);

    fn unbond(env: Env, sender: Address, stake_amount: i128, stake_timestamp: u64);

    fn distribute_rewards(env: Env);

    fn withdraw_rewards(env: Env, sender: Address);

    // ADMIN
    fn stake_rewards_add_users(env: Env, users: Vec<Address>);

    // QUERIES

    fn query_config(env: Env) -> ConfigResponse;

    fn query_admin(env: Env) -> Address;

    fn query_staked(env: Env, address: Address) -> StakedResponse;

    fn query_total_staked(env: Env) -> i128;

    fn query_annualized_rewards(env: Env) -> Val;

    fn query_withdrawable_rewards(env: Env, address: Address) -> WithdrawableRewardResponse;

    fn query_distributed_rewards(env: Env, asset: Address) -> u128;

    fn query_undistributed_rewards(env: Env, asset: Address) -> u128;
}

#[contractimpl]
impl StakingTrait for Staking {
    #[allow(clippy::too_many_arguments)]
    fn initialize(
        env: Env,
        admin: Address,
        lp_token: Address,
        stake_rewards: Address,
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
        set_stake_rewards(&env, &stake_rewards);
    }

    fn bond(env: Env, sender: Address, tokens: i128) {
        sender.require_auth();

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

        // Call stake_rewards contract to calculate for the rewards
        let bond_fn_arg: Vec<Val> = (sender.clone(), stakes.clone()).into_val(&env);
        env.invoke_contract::<Val>(
            &get_stake_rewards(&env),
            &Symbol::new(&env, "calculate_bond"),
            bond_fn_arg,
        );

        env.events().publish(("bond", "user"), &sender);
        env.events().publish(("bond", "token"), &config.lp_token);
        env.events().publish(("bond", "amount"), tokens);
    }

    fn unbond(env: Env, sender: Address, stake_amount: i128, stake_timestamp: u64) {
        sender.require_auth();

        let config = get_config(&env);

        // check for rewards and withdraw them
        let found_rewards: WithdrawableRewardResponse =
            Self::query_withdrawable_rewards(env.clone(), sender.clone());

        if !found_rewards.reward_amount == 0 {
            Self::withdraw_rewards(env.clone(), sender.clone());
        }

        let mut stakes = get_stakes(&env, &sender);

        // Call stake_rewards contract to update the reward calculations
        let unbond_fn_arg: Vec<Val> = (sender.clone(), stakes.clone()).into_val(&env);
        env.invoke_contract::<Val>(
            &get_stake_rewards(&env),
            &Symbol::new(&env, "calculate_unbond"),
            unbond_fn_arg,
        );

        remove_stake(&env, &mut stakes.stakes, stake_amount, stake_timestamp);
        stakes.total_stake -= stake_amount;

        let lp_token_client = token_contract::Client::new(&env, &config.lp_token);
        lp_token_client.transfer(&env.current_contract_address(), &sender, &stake_amount);

        save_stakes(&env, &sender, &stakes);
        utils::decrease_total_staked(&env, &stake_amount);

        env.events().publish(("unbond", "user"), &sender);
        env.events().publish(("bond", "token"), &config.lp_token);
        env.events().publish(("bond", "amount"), stake_amount);
    }

    fn distribute_rewards(env: Env) {
        let stakes = get_total_staked_counter(&env);
        // Call stake_rewards contract to update the reward calculations
        let distr_fn_arg: Val = stakes.into_val(&env);
        env.invoke_contract::<Val>(
            &get_stake_rewards(&env),
            &Symbol::new(&env, "distribute_rewards"),
            vec![&env, distr_fn_arg],
        );
    }

    fn withdraw_rewards(env: Env, sender: Address) {
        let stakes = get_stakes(&env, &sender);
        let withdraw_fn_arg: Vec<Val> = (sender, stakes).into_val(&env);
        env.invoke_contract::<Val>(
            &get_stake_rewards(&env),
            &Symbol::new(&env, "withdraw_rewards"),
            withdraw_fn_arg,
        );
    }

    fn stake_rewards_add_users(env: Env, users: Vec<Address>) {
        for user in users {
            let stakes = get_stakes(&env, &user);
            // Call stake_rewards contract to update the reward calculations
            let unbond_fn_arg: Vec<Val> = (user, stakes).into_val(&env);
            env.invoke_contract::<Val>(
                &get_stake_rewards(&env),
                &Symbol::new(&env, "add_user"),
                unbond_fn_arg,
            );
        }
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
        let stakes = get_stakes(&env, &address);
        StakedResponse {
            stakes: stakes.stakes,
            total_stake: stakes.total_stake,
        }
    }

    fn query_total_staked(env: Env) -> i128 {
        get_total_staked_counter(&env)
    }

    fn query_annualized_rewards(env: Env) -> Val {
        let stakes = get_total_staked_counter(&env);
        let apr_fn_arg: Val = stakes.into_val(&env);
        let ret: Val = env.invoke_contract::<Val>(
            &get_stake_rewards(&env),
            &Symbol::new(&env, "query_annualized_reward"),
            vec![&env, apr_fn_arg],
        );
        ret
    }

    fn query_withdrawable_rewards(env: Env, user: Address) -> WithdrawableRewardResponse {
        let stakes = get_stakes(&env, &user);
        let apr_fn_arg: Val = stakes.into_val(&env);
        let ret: WithdrawableRewardResponse = env.invoke_contract(
            &get_stake_rewards(&env),
            &Symbol::new(&env, "query_withdrawable_reward"),
            vec![&env, apr_fn_arg],
        );
        ret
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
impl Staking {
    #[allow(dead_code)]
    pub fn update(env: Env, new_wasm_hash: BytesN<32>, staking_rewards: Address) {
        let admin = get_admin(&env);
        admin.require_auth();

        set_stake_rewards(&env, &staking_rewards);

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
