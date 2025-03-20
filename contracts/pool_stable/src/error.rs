use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    SpreadExceedsLimit = 300,
    ProvideLiquiditySlippageToleranceTooHigh = 301,
    WithdrawLiquidityMinimumAmountOfAOrBIsNotSatisfied = 302,
    ValidateFeeBpsTotalFeesCantBeGreaterThan100 = 303,
    TotalSharesEqualZero = 304,
    AssetNotInPool = 305,
    AlreadyInitialized = 306,
    TokenABiggerThanTokenB = 307,
    InvalidBps = 308,
    LowLiquidity = 309,
    Unauthorized = 310,
    IncorrectAssetSwap = 311,
    NewtonMethodFailed = 312,
    CalcYErr = 313,
    SwapMinReceivedBiggerThanReturn = 314,
    ProvideLiquidityBothTokensMustBeMoreThanZero = 315,
    DivisionByZero = 316,
    InvalidAMP = 317,
    TransactionAfterTimestampDeadline = 318,
    SlippageToleranceExceeded = 319,
    IssuedSharesLessThanUserRequested = 320,
    SwapFeeBpsOverLimit = 321,
    UserDeclinesPoolFee = 322,
    AdminNotSet = 323,
    ContractMathError = 324,
    NegativeInputProvided = 325,
    SameAdmin = 326,
    NoAdminChangeInPlace = 327,
    AdminChangeExpired = 328,
}
