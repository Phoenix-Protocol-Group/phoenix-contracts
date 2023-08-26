// use soroban_sdk::contracterror;
//
// #[contracterror]
// #[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
// #[repr(u32)]
// pub enum ContractError {
//     /// Initialization errors
//     ArgumentsInvalidLessOrEqualZero = 1,
//     TokenPerPowerCannotBeZero = 2,
//
//     /// Reward errors
//     MinRewardTooSmall = 3,
//     MinRewardNotReached = 4,
//     NoRewardsForThisAsset = 5,
//     FundDistributionStartTimeTooEarly = 6,
//     RewardsValidationFailed = 7,
//     DistributionAlreadyAdded = 8,
//     WithdrawAdjustmentMissing = 9,
//     DistributionNotFound = 10,
//     RewardsNotDistributedOrDistributionNotCreated = 11,
//
//     /// Stake errors
//     MinStakeLessOrEqualZero = 12,
//     StakeLessThenMinBond = 13,
//     StakeNotFound = 14,
//     TotalStakedCannotBeZeroOrLess = 15,
//
//     /// Storage errors
//     ConfigNotSet = 16,
//     FailedToGetAdminAddrFromStorage = 17,
//
//     FirstTokenMustBeSmallerThenSecond = 18,
//     InvalidFeeBps = 19,
//
//     /// Swap errors
//     SlippageToleranceExceeded = 20,
//     SlippageToleranceViolated = 21,
//     SpreadExceedsMaxAllowed = 22,
//     EmptyPoolBalance = 23,
//
//     // Storage errors
//     FailedToLoadFromStorage = 24,
//     IncorrectLiquidityParametersForA = 25,
//     IncorrectLiquidityParametersForB = 26,
//     FailedToGetTotalSharesFromStorage = 27,
//     FailedToGetPoolBalanceAFromStorage = 28,
//     FailedToGetPoolBalanceBFromStorage = 29,
//     DepositAmountAExceedsDesired = 30,
//     DepositAmountBelowMinA = 31,
//     DepositAmountBExceedsDesired = 32,
//     DepositAmountBelowMinB = 33,
//
//     /// Liquidity errors
//     WithdrawMinNotSatisfied = 34,
//     InvalidAmounts = 35,
//
//     /// Other errors
//     Unauthorized = 36,
// }
