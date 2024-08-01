use soroban_sdk::{contracttype, Address, String, Vec};

use crate::storage::{Config, Stake};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigResponse {
    pub config: Config,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StakedResponse {
    pub stakes: Vec<Stake>,
    pub total_stake: i128,
}

#[contracttype]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AnnualizedReward {
    pub asset: Address,
    pub amount: String,
}

#[contracttype]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AnnualizedRewardsResponse {
    pub rewards: Vec<AnnualizedReward>,
}
#[contracttype]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WithdrawableReward {
    pub reward_address: Address,
    pub reward_amount: u128,
}

#[contracttype]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WithdrawableRewardResponse {
    pub reward_address: Address,
    pub reward_amount: u128,
}
