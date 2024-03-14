use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    AlreadyInitialized = 1,
    InvalidMinBond = 2,
    InvalidMinReward = 3,
    InvalidBond = 4,
    Unauthorized = 5,
    MinRewardNotEnough = 6,
    RewardsInvalid = 7,
    StakeNotFound = 8,
    InvalidTime = 9,
    DistributionExists = 10,
}
