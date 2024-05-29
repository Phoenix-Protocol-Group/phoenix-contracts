use soroban_sdk::{contracttype, Address, String};

use crate::storage::Config;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigResponse {
    pub config: Config,
}

#[contracttype]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AnnualizedRewardResponse {
    pub asset: Address,
    pub amount: String,
}

#[contracttype]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WithdrawableRewardResponse {
    pub reward_address: Address,
    pub reward_amount: u128,
}
