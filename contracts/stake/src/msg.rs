use soroban_sdk::{contracttype, Address, Vec};

use crate::{
    storage::{Config, Stake},
    utils::OptionUint,
};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigResponse {
    pub config: Config,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StakedResponse {
    pub stakes: Vec<Stake>,
}

#[contracttype]
pub struct AnnualizedRewardsResponse {
    info: Address,
    /// None means contract does not know the value - total_staked or total_power could be 0.
    amount: OptionUint,
}

#[contracttype]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WithdrawableReward {
    pub reward_address: Address,
    pub reward_amount: u128,
}

#[contracttype]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WithdrawableRewardsResponse {
    /// Amount of rewards assigned for withdrawal from the given address.
    pub rewards: Vec<WithdrawableReward>,
}
