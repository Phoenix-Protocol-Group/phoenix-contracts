use soroban_sdk::{contracttype, Address, String, Vec};

use crate::storage::Config;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigResponse {
    pub config: Config,
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
pub struct WithdrawableRewardsResponse {
    /// Amount of rewards assigned for withdrawal from the given address.
    pub rewards: Vec<WithdrawableReward>,
}
