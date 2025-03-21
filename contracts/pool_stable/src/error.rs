use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    SpreadExceedsLimit = 400,
    ProvideLiquiditySlippageToleranceTooHigh = 401,
    WithdrawLiquidityMinimumAmountOfAOrBIsNotSatisfied = 402,
    ValidateFeeBpsTotalFeesCantBeGreaterThan100 = 403,
    TotalSharesEqualZero = 404,
    AssetNotInPool = 405,
    AlreadyInitialized = 406,
    TokenABiggerThanTokenB = 407,
    InvalidBps = 408,
    LowLiquidity = 409,
    Unauthorized = 410,
    IncorrectAssetSwap = 411,
    NewtonMethodFailed = 412,
    CalcYErr = 413,
    SwapMinReceivedBiggerThanReturn = 414,
    ProvideLiquidityBothTokensMustBeMoreThanZero = 415,
    DivisionByZero = 416,
    InvalidAMP = 417,
    TransactionAfterTimestampDeadline = 418,
    SlippageToleranceExceeded = 419,
    IssuedSharesLessThanUserRequested = 420,
    SwapFeeBpsOverLimit = 421,
    UserDeclinesPoolFee = 422,
    AdminNotSet = 423,
    ContractMathError = 424,
    NegativeInputProvided = 425,
    SameAdmin = 426,
    NoAdminChangeInPlace = 427,
    AdminChangeExpired = 428,
}
