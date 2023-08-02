use soroban_sdk::{contract, contractimpl, contractmeta, log, Address, Env, Vec};

use crate::{
    error::ContractError,
    msg::{AnnualizedRewardsResponse, ConfigResponse, StakedResponse},
    storage::{
        get_config, get_stakes, save_config, save_stakes,
        utils::{self, get_admin},
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

pub trait StakingTrait {
    // Sets the token contract addresses for this pool
    // epoch: Number of seconds between payments
    fn initialize(
        env: Env,
        admin: Address,
        lp_token: Address,
        token_per_power: u128,
        min_bond: i128,
        max_distributions: u32,
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
        manager: Address,
        asset: Address,
    ) -> Result<(), ContractError>;

    fn distribute_rewards(env: Env) -> Result<(), ContractError>;

    fn withdraw_rewards(env: Env, receiver: Option<Address>) -> Result<(), ContractError>;

    fn fund_distribution(
        env: Env,
        start_time: u64,
        distribution_duration: u64,
        amount: u128,
    ) -> Result<(), ContractError>;

    // QUERIES

    fn query_config(env: Env) -> Result<ConfigResponse, ContractError>;

    fn query_admin(env: Env) -> Result<Address, ContractError>;

    fn query_staked(env: Env, address: Address) -> Result<StakedResponse, ContractError>;

    fn query_annualized_rewards(env: Env) -> Result<AnnualizedRewardsResponse, ContractError>;

    fn query_withdrawable_rewards(env: Env, address: Address) -> Result<(), ContractError>;

    fn query_distributed_rewards(env: Env) -> Result<(), ContractError>;
}

#[contractimpl]
impl StakingTrait for Staking {
    fn initialize(
        env: Env,
        admin: Address,
        lp_token: Address,
        token_per_power: u128,
        min_bond: i128,
        max_distributions: u32,
    ) -> Result<(), ContractError> {
        if min_bond <= 0 {
            log!(
                &env,
                "Minimum amount of lp share tokens to bond can not be smaller or equal to 0"
            );
            return Err(ContractError::MinStakeLessOrEqualZero);
        }
        if token_per_power == 0 {
            log!(
                &env,
                "Token per power set as 0 - this would break staking rewards!"
            );
            return Err(ContractError::TokenPerPowerCannotBeZero);
        }

        env.events()
            .publish(("initialize", "LP Share token staking contract"), &lp_token);

        let config = Config {
            lp_token,
            token_per_power,
            min_bond,
            max_distributions,
        };
        save_config(&env, config);

        utils::save_admin(&env, &admin);

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
        // TODO: Discuss: Add implementation to add stake if another is present in +-24h timestamp to avoid
        // creating multiple stakes the same day

        stakes.stakes.push_back(stake);
        save_stakes(&env, &sender, &stakes);

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

        let lp_token_client = token_contract::Client::new(&env, &config.lp_token);
        lp_token_client.transfer(&env.current_contract_address(), &sender, &stake_amount);

        save_stakes(&env, &sender, &stakes);

        env.events().publish(("unbond", "user"), &sender);
        env.events().publish(("bond", "token"), &config.lp_token);
        env.events().publish(("bond", "amount"), stake_amount);

        Ok(())
    }

    fn create_distribution_flow(
        _env: Env,
        _manager: Address,
        _asset: Address,
    ) -> Result<(), ContractError> {
        unimplemented!();
    }

    fn distribute_rewards(_env: Env) -> Result<(), ContractError> {
        unimplemented!();
    }

    fn withdraw_rewards(_env: Env, _receiver: Option<Address>) -> Result<(), ContractError> {
        unimplemented!();
    }

    fn fund_distribution(
        _env: Env,
        _start_time: u64,
        _distribution_duration: u64,
        _amount: u128,
    ) -> Result<(), ContractError> {
        unimplemented!();
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

    fn query_annualized_rewards(_env: Env) -> Result<AnnualizedRewardsResponse, ContractError> {
        unimplemented!();
    }

    fn query_withdrawable_rewards(_env: Env, _address: Address) -> Result<(), ContractError> {
        unimplemented!();
    }

    fn query_distributed_rewards(_env: Env) -> Result<(), ContractError> {
        unimplemented!();
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
