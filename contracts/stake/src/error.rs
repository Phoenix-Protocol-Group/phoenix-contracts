use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    AlreadyInitialized = 400,
    InvalidMinBond = 401,
    InvalidMinReward = 402,
    InvalidBond = 403,
    Unauthorized = 404,
    MinRewardNotEnough = 405,
    RewardsInvalid = 406,
    StakeNotFound = 409,
    InvalidTime = 410,
    DistributionExists = 411,
    InvalidRewardAmount = 412,
    InvalidMaxComplexity = 413,
    DistributionNotFound = 414,
    AdminNotSet = 415,
    ContractMathError = 416,
    RewardCurveDoesNotExist = 417,
    SameAdmin = 418,
    NoAdminChangeInPlace = 419,
    AdminChangeExpired = 420,
}
