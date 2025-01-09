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
    TokenABiggerThanTokenB = 7,
    InvalidBps = 8,
    LowLiquidity = 9,
    Unauthorized = 10,
    IncorrectAssetSwap = 11,
    NewtonMethodFailed = 12,
    CalcYErr = 13,
    SwapMinReceivedBiggerThanReturn = 14,
    ProvideLiquidityBothTokensMustBeMoreThanZero = 15,
    DivisionByZero = 16,
    InvalidAMP = 17,
    TransactionAfterTimestampDeadline = 18,
    SlippageToleranceExceeded = 19,
    IssuedSharesLessThanUserRequested = 20,
    SwapFeeBpsOverLimit = 21,
    UserDeclinesPoolFee = 22,
    AdminNotSet = 23,
}
