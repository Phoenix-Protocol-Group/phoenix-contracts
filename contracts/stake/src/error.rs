use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    InvalidMinBond = 1,
    InvalidMinReward = 2,
    InvalidBond = 3,
    Unauthorized = 4,
    MinRewardNotEnough = 5,
    RewardsInvalid = 6,
    StakeNotFound = 7,
    InvalidTime = 8,
    DistributionExists = 9,
    InvalidRewardAmount = 10,
    InvalidMaxComplexity = 11,
    DistributionNotFound = 12,
    AdminNotSet = 13,
}
