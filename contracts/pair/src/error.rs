use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SwapError {
    SlippageToleranceExceeded = 1,
    SlippageToleranceViolated = 2,
    SpreadExceedsMaxAllowed = 3,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    FirstTokenMustBeSmallerThenSecond = 1,
    Swapage(SwapError) = 2,
    ConfigNotSet = 3,
    FailedToLoadFromStorage = 4,
    IncorrectLiqudityParameters = 5,
    DepositAmountExceedsOrBelowMin = 6,
    WithdrawMinNotSatisfied = 7,
    InvalidFeeBps = 8,
    EmptyPoolBalance = 9,
    InvalidAmounts = 10,
    Unauthorized = 11,
    ArgumentsInvalidLessOrEqualZero = 12,
}
