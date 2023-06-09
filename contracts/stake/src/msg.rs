use soroban_sdk::{contracttype, Address, Vec};

use crate::utils::OptionUint;

#[contracttype]
pub struct StakedResponse {
    stake: u128,
}

#[contracttype]
pub struct AllStakedResponse {
    stakes: Vec<(Address, StakedResponse)>,
}

#[contracttype]
pub struct AnnualizedRewardsResponse {
    info: Address,
    /// None means contract does not know the value - total_staked or total_power could be 0.
    amount: OptionUint,
}

#[contracttype]
pub struct WithdrawableRewardsResponse {
    /// Amount of rewards assigned for withdrawal from the given address.
    rewards: Vec<(Address, u128)>,
}

#[contracttype]
pub struct DistributedRewardsResponse {
    /// Total number of tokens sent to the contract over all time.
    distributed: Vec<(Address, u128)>,
    /// Total number of tokens available to be withdrawn.
    withdrawable: Vec<(Address, u128)>,
}
