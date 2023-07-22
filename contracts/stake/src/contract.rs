use soroban_sdk::{contractimpl, contractmeta, log, Address, Env};

use crate::{
    error::ContractError,
    msg::{
        AllStakedResponse, AnnualizedRewardsResponse, ConfigResponse, DistributedRewardsResponse,
        StakedResponse, WithdrawableRewardsResponse,
    },
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

    fn unbond(env: Env, tokens: u128) -> Result<(), ContractError>;

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

    fn query_all_staked(env: Env) -> Result<AllStakedResponse, ContractError>;

    fn query_annualized_rewards(env: Env) -> Result<AnnualizedRewardsResponse, ContractError>;

    fn query_withdrawable_rewards(
        env: Env,
        address: Address,
    ) -> Result<WithdrawableRewardsResponse, ContractError>;

    fn query_distributed_rewards(env: Env) -> Result<DistributedRewardsResponse, ContractError>;
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

    fn unbond(_env: Env, _tokens: u128) -> Result<(), ContractError> {
        unimplemented!();
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

    fn query_staked(_env: Env, _address: Address) -> Result<StakedResponse, ContractError> {
        unimplemented!();
    }

    fn query_all_staked(_env: Env) -> Result<AllStakedResponse, ContractError> {
        unimplemented!();
    }

    fn query_annualized_rewards(_env: Env) -> Result<AnnualizedRewardsResponse, ContractError> {
        unimplemented!();
    }

    fn query_withdrawable_rewards(
        _env: Env,
        _address: Address,
    ) -> Result<WithdrawableRewardsResponse, ContractError> {
        unimplemented!();
    }

    fn query_distributed_rewards(_env: Env) -> Result<DistributedRewardsResponse, ContractError> {
        unimplemented!();
    }
}
