use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    SpreadExceedsLimit = 300,

    ProvideLiquiditySlippageToleranceTooHigh = 301,
    ProvideLiquidityAtLeastOneTokenMustBeBiggerThenZero = 302,

    WithdrawLiquidityMinimumAmountOfAOrBIsNotSatisfied = 303,
    SplitDepositBothPoolsAndDepositMustBePositive = 304,
    ValidateFeeBpsTotalFeesCantBeGreaterThan100 = 305,

    GetDepositAmountsMinABiggerThenDesiredA = 306,
    GetDepositAmountsMinBBiggerThenDesiredB = 307,
    GetDepositAmountsAmountABiggerThenDesiredA = 308,
    GetDepositAmountsAmountALessThenMinA = 309,
    GetDepositAmountsAmountBBiggerThenDesiredB = 310,
    GetDepositAmountsAmountBLessThenMinB = 311,
    TotalSharesEqualZero = 312,
    DesiredAmountsBelowOrEqualZero = 313,
    MinAmountsBelowZero = 314,
    AssetNotInPool = 315,
    AlreadyInitialized = 316,
    TokenABiggerThanTokenB = 317,
    InvalidBps = 318,
    SlippageInvalid = 319,

    SwapMinReceivedBiggerThanReturn = 320,
    TransactionAfterTimestampDeadline = 321,
    CannotConvertU256ToI128 = 322,
    UserDeclinesPoolFee = 323,
    SwapFeeBpsOverLimit = 324,
    NotEnoughSharesToBeMinted = 325,
    NotEnoughLiquidityProvided = 326,
    AdminNotSet = 327,
    ContractMathError = 328,
    NegativeInputProvided = 329,
    SameAdmin = 330,
    NoAdminChangeInPlace = 331,
    AdminChangeExpired = 332,
}
