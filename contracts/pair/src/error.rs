use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    /// Initialization errors
    FirstTokenMustBeSmallerThenSecond = 1,
    InvalidFeeBps = 2,

    /// Swap errors
    SlippageToleranceExceeded = 3,
    SlippageToleranceViolated = 4,
    SpreadExceedsMaxAllowed = 5,
    EmptyPoolBalance = 6,

    // Storage errors
    ConfigNotSet = 7,
    FailedToLoadFromStorage = 8,
    IncorrectLiquidityParametersForA = 9,
    IncorrectLiquidityParametersForB = 10,
    FailedToGetAdminAddrFromStorage = 11,
    FailedToGetTotalSharesFromStorage = 12,
    FailedToGetPoolBalanceAFromStorage = 13,
    FailedToGetPoolBalanceBFromStorage = 14,
    DepositAmountAExceedsDesired = 15,
    DepositAmountBelowMinA = 16,
    DepositAmountBExceedsDesired = 17,
    DepositAmountBelowMinB = 18,

    /// Liquidity errors
    WithdrawMinNotSatisfied = 19,
    InvalidAmounts = 20,

    /// Other errors
    Unauthorized = 21,
    ArgumentsInvalidLessOrEqualZero = 22,
}
