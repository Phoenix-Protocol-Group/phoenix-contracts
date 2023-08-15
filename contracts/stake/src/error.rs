use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    /// Initialization errors
    TokenPerPowerCannotBeZero = 1,

    /// Reward errors
    MinRewardTooSmall = 2,
    MinRewardNotReached = 3,
    NoRewardsForThisAsset = 4,
    FundDistributionStartTimeTooEarly = 12,
    RewardsValidationFailed = 13,
    DistributionAlreadyAdded = 14,
    WithdrawAdjustmentMissing = 15,

    /// Stake errros
    MinStakeLessOrEqualZero = 5,
    StakeLessThenMinBond = 6,
    StakeNotFound = 7,
    TotalStakedCannotBeZeroOrLess = 8,

    /// Storage errors
    ConfigNotSet = 9,
    FailedToGetAdminAddrFromStorage = 10,

    /// Other errors
    Unauthorized = 11,
}
