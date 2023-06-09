use soroban_sdk::{contractimpl, contractmeta, Address, Env};

use crate::{
    error::ContractError,
    msg::{
        AllStakedResponse, AnnualizedRewardsResponse, DistributedRewardsResponse, StakedResponse,
        WithdrawableRewardsResponse,
    },
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
        min_bond: u128,
        max_distributions: u32,
    ) -> Result<(), ContractError>;

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
        _env: Env,
        _admin: Address,
        _lp_token: Address,
        _token_per_power: u128,
        _min_bond: u128,
        _max_distributions: u32,
    ) -> Result<(), ContractError> {
        unimplemented!();
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
