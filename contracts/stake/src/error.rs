use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    AlreadyInitialized = 500,
    InvalidMinBond = 501,
    InvalidMinReward = 502,
    InvalidBond = 503,
    Unauthorized = 504,
    MinRewardNotEnough = 505,
    RewardsInvalid = 506,
    StakeNotFound = 509,
    InvalidTime = 510,
    DistributionExists = 511,
    InvalidRewardAmount = 512,
    InvalidMaxComplexity = 513,
    DistributionNotFound = 514,
    AdminNotSet = 515,
    ContractMathError = 516,
    RewardCurveDoesNotExist = 517,
    SameAdmin = 518,
    NoAdminChangeInPlace = 519,
    AdminChangeExpired = 520,
}
