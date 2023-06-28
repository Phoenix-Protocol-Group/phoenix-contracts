use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    FirstTokenMustBeSmallerThenSecond = 1,
    SlippageToleranceExceeded = 2,
    SlippageToleranceViolated = 3,
    SpreadExceedsMaxAllowed = 4,
    ConfigNotSet = 5,
    FailedToLoadFromStorage = 6,
    DepositAmountBLessThenMin = 7,
    DepositAmountAExceedsOrBelowMin = 8,
    WithdrawMinNotSatisfied = 9,
    InvalidFeeBps = 11,
    EmptyPoolBalance = 12,
    InvalidAmounts = 13,
    Unauthorized = 14,
}
