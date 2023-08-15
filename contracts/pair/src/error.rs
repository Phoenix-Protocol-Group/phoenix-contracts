use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    /// Swap errors
    SlippageToleranceExceeded = 2,
    SlippageToleranceViolated = 3,
    SpreadExceedsMaxAllowed = 4,
    EmptyPoolBalance = 11,

    /// Initialization errors
    FirstTokenMustBeSmallerThenSecond = 1,
    InvalidFeeBps = 10,

    // Storage errors
    ConfigNotSet = 5,
    FailedToLoadFromStorage = 6,
    IncorrectLiqudityParameters = 7,
    DepositAmountExceedsOrBelowMin = 8,

    /// Liquidity errors
    WithdrawMinNotSatisfied = 9,
    InvalidAmounts = 12,

    /// Other errors
    Unauthorized = 13,
    ArgumentsInvalidLessOrEqualZero = 14,
}
