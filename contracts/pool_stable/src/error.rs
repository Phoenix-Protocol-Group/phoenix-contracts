use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    SpreadExceedsLimit = 1,
    ProvideLiquiditySlippageToleranceTooHigh = 2,
    WithdrawLiquidityMinimumAmountOfAOrBIsNotSatisfied = 3,
    ValidateFeeBpsTotalFeesCantBeGreaterThan100 = 4,
    TotalSharesEqualZero = 5,
    AssetNotInPool = 6,
    AlreadyInitialized = 7,
    TokenABiggerThanTokenB = 8,
    InvalidBps = 9,
    LowLiquidity = 10,
    Unauthorized = 11,
    IncorrectAssetSwap = 12,
    NewtonMethodFailed = 13,
    CalcYErr = 14,
    SwapMinReceivedBiggerThanReturn = 15,
    ProvideLiquidityBothTokensMustBeMoreThanZero = 16,
    DivisionByZero = 17,
    InvalidAMP = 18,
    TransactionAfterTimestampDeadline = 19,
    SlippageToleranceExceeded = 20,
    IssuedSharesLessThanUserRequested = 21,
    SwapFeeBpsOverLimit = 22,
    UserDeclinesPoolFee = 23,
    AdminNotSet = 24,
    ContractMathError = 25,
    InvalidNumberOfTokenDecimals = 26,
}
