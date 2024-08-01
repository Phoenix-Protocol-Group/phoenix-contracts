use phoenix::utils::{convert_i128_to_u128, convert_u128_to_i128};
use soroban_decimal::Decimal;
use soroban_sdk::{
    contract, contractimpl, contractmeta, log, panic_with_error, vec, Address, BytesN, Env,
    IntoVal, String, Symbol, Val, Vec,
};

use crate::{
    distribution::{
        calc_power, calculate_annualized_payout, get_distribution, get_reward_curve,
        get_withdraw_adjustment, save_distribution, save_reward_curve, save_withdraw_adjustment,
        update_rewards, withdrawable_rewards, Distribution, SHARES_SHIFT,
    },
    error::ContractError,
    msg::{
        AnnualizedReward, AnnualizedRewardsResponse, ConfigResponse, StakedResponse,
        WithdrawableReward, WithdrawableRewardsResponse,
    },
    storage::{
        get_config, get_stakes, save_config, save_stakes,
        utils::{
            self, add_distribution, find_stake_rewards_by_asset, get_admin, get_distributions,
            get_stake_rewards, get_total_staked_counter, is_initialized, set_initialized,
            set_stake_rewards,
        },
        Config, Stake,
    },
    token_contract, TOKEN_PER_POWER,
};
use curve::Curve;

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
        stake_rewards: BytesN<32>,
        min_bond: i128,
        min_reward: i128,
        manager: Address,
        owner: Address,
        max_complexity: u32,
    );

    fn bond(env: Env, sender: Address, tokens: i128);

    fn unbond(env: Env, sender: Address, stake_amount: i128, stake_timestamp: u64);

    fn create_distribution_flow(
        env: Env,
        sender: Address,
        asset: Address,
        salt: BytesN<32>,
        max_complexity: u32,
        min_reward: u128,
        min_bond: u128,
    );

    fn distribute_rewards(env: Env);

    fn withdraw_rewards(env: Env, sender: Address);

    fn fund_distribution(
        env: Env,
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

    // ADMIN
    fn stake_rewards_add_users(env: Env, staking_rewards: Address, users: Vec<Address>);
}

#[contractimpl]
impl StakingTrait for Staking {
    #[allow(clippy::too_many_arguments)]
    fn initialize(
        env: Env,
        admin: Address,
        lp_token: Address,
        stake_rewards: BytesN<32>,
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

        for (_asset, distribution_address) in get_distributions(&env) {
            // Call stake_rewards contract to calculate for the rewards
            let bond_fn_arg: Vec<Val> = (sender.clone(), stakes.clone()).into_val(&env);
            env.invoke_contract::<Val>(
                &distribution_address,
                &Symbol::new(&env, "calculate_bond"),
                bond_fn_arg,
            );
        }

        save_stakes(&env, &sender, &stakes);
        utils::increase_total_staked(&env, &tokens);

        env.events().publish(("bond", "user"), &sender);
        env.events().publish(("bond", "token"), &config.lp_token);
        env.events().publish(("bond", "amount"), tokens);
    }

    fn unbond(env: Env, sender: Address, stake_amount: i128, stake_timestamp: u64) {
        sender.require_auth();

        let config = get_config(&env);

        // check for rewards and withdraw them
        let found_rewards: WithdrawableRewardsResponse =
            Self::query_withdrawable_rewards(env.clone(), sender.clone());

        if !found_rewards.rewards.is_empty() {
            Self::withdraw_rewards(env.clone(), sender.clone());
        }

        let mut stakes = get_stakes(&env, &sender);

        for (_asset, distribution_address) in get_distributions(&env) {
            // Call stake_rewards contract to update the reward calculations
            let unbond_fn_arg: Vec<Val> = (sender.clone(), stakes.clone()).into_val(&env);
            env.invoke_contract::<Val>(
                &distribution_address,
                &Symbol::new(&env, "calculate_unbond"),
                unbond_fn_arg,
            );
        }

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

    fn create_distribution_flow(
        env: Env,
        sender: Address,
        asset: Address,
        salt: BytesN<32>,
        max_complexity: u32,
        min_reward: u128,
        min_bond: u128,
    ) {
        sender.require_auth();

        let manager = get_config(&env).manager;
        let owner = get_config(&env).owner;
        if sender != manager && sender != owner {
            log!(env, "Stake: create distribution: Non-authorized creation!");
            panic_with_error!(&env, ContractError::Unauthorized);
        }
        let deployed_stake_rewards = env
            .deployer()
            .with_address(sender, salt)
            .deploy(get_stake_rewards(&env));

        let init_fn = Symbol::new(&env, "initialize");
        let init_fn_args: Vec<Val> =
            (owner, asset.clone(), max_complexity, min_reward, min_bond).into_val(&env);
        let _: Val = env.invoke_contract(&deployed_stake_rewards, &init_fn, init_fn_args);

        add_distribution(&env, &asset, &deployed_stake_rewards);

        env.events().publish(
            ("create_distribution_flow", "asset"),
            &deployed_stake_rewards,
        );
    }

    fn distribute_rewards(env: Env) {
        let total_staked_amount = get_total_staked_counter(&env);
        let total_rewards_power_result = calc_power(
            &get_config(&env),
            total_staked_amount,
            Decimal::one(),
            TOKEN_PER_POWER,
        );
        let total_rewards_power = convert_i128_to_u128(total_rewards_power_result);

        if total_rewards_power == 0 {
            log!(&env, "Stake: No rewards to distribute!");
            return;
        }
        let stakes = get_total_staked_counter(&env);
        for (asset, distribution_address) in get_distributions(&env) {
            // Call stake_rewards contract to update the reward calculations
            let distr_fn_arg: Val = stakes.into_val(&env);
            env.invoke_contract::<Val>(
                &distribution_address,
                &Symbol::new(&env, "distribute_rewards"),
                vec![&env, distr_fn_arg],
            );

            env.events()
                .publish(("distribute_rewards", "asset"), &asset);
        }
    }

    fn withdraw_rewards(env: Env, sender: Address) {
        env.events().publish(("withdraw_rewards", "user"), &sender);
        let stakes = get_stakes(&env, &sender);

        for (asset, distribution_address) in get_distributions(&env) {
            let withdraw_fn_arg: Vec<Val> = (sender.clone(), stakes.clone()).into_val(&env);
            env.invoke_contract::<Val>(
                &distribution_address,
                &Symbol::new(&env, "withdraw_rewards"),
                withdraw_fn_arg,
            );

            env.events()
                .publish(("withdraw_rewards", "reward_token"), &asset);
        }
    }

    fn fund_distribution(
        env: Env,
        start_time: u64,
        distribution_duration: u64,
        token_address: Address,
        token_amount: i128,
    ) {
        let admin = get_admin(&env);
        admin.require_auth();

        let fund_distr_fn_arg: Vec<Val> =
            (start_time, distribution_duration, token_amount.clone()).into_val(&env);
        env.invoke_contract::<Val>(
            &find_stake_rewards_by_asset(&env, &token_address).unwrap(),
            &Symbol::new(&env, "fund_distribution"),
            fund_distr_fn_arg,
        );

        env.events()
            .publish(("fund_reward_distribution", "asset"), &token_address);
        env.events()
            .publish(("fund_reward_distribution", "amount"), token_amount);
        env.events()
            .publish(("fund_reward_distribution", "start_time"), start_time);
        env.events().publish(
            ("fund_reward_distribution", "end_time"),
            start_time + distribution_duration,
        );
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

    fn query_annualized_rewards(env: Env) -> AnnualizedRewardsResponse {
        let mut aprs = vec![&env];
        let total_stake_amount = get_total_staked_counter(&env);
        let apr_fn_arg: Val = total_stake_amount.into_val(&env);

        for (_asset, distribution_address) in get_distributions(&env) {
            let apr: AnnualizedReward = env.invoke_contract(
                &distribution_address,
                &Symbol::new(&env, "query_annualized_reward"),
                vec![&env, apr_fn_arg],
            );

            aprs.push_back(AnnualizedReward {
                asset: distribution_address.clone(),
                amount: apr.amount,
            });
        }

        AnnualizedRewardsResponse { rewards: aprs }
    }

    fn query_withdrawable_rewards(env: Env, user: Address) -> WithdrawableRewardsResponse {
        let stakes = get_stakes(&env, &user);
        // iterate over all distributions and calculate withdrawable rewards
        let mut rewards = vec![&env];
        for (_asset, distribution_address) in get_distributions(&env) {
            let apr_fn_arg: Val = stakes.into_val(&env);
            let ret: WithdrawableReward = env.invoke_contract(
                &distribution_address,
                &Symbol::new(&env, "query_withdrawable_reward"),
                vec![&env, apr_fn_arg],
            );

            rewards.push_back(WithdrawableReward {
                reward_address: distribution_address,
                reward_amount: ret.reward_amount,
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
        let reward_token_balance = reward_token_client.balance(&env.current_contract_address());
        convert_i128_to_u128(reward_token_balance) - distribution.withdrawable_total
    }

    fn stake_rewards_add_users(env: Env, staking_contract: Address, users: Vec<Address>) {
        for user in users {
            let stakes = get_stakes(&env, &user);
            // Call stake_rewards contract to update the reward calculations
            let add_user_fn_arg: Vec<Val> = (user, stakes).into_val(&env);
            env.invoke_contract::<Val>(
                &staking_contract,
                &Symbol::new(&env, "add_user"),
                add_user_fn_arg,
            );
        }
    }
}

#[contractimpl]
impl Staking {
    #[allow(dead_code)]
    pub fn update(env: Env, new_wasm_hash: BytesN<32>, staking_rewards: BytesN<32>) {
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
