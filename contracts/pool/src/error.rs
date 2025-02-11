use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    SpreadExceedsLimit = 200,

    ProvideLiquiditySlippageToleranceTooHigh = 201,
    ProvideLiquidityAtLeastOneTokenMustBeBiggerThenZero = 202,

    WithdrawLiquidityMinimumAmountOfAOrBIsNotSatisfied = 203,
    SplitDepositBothPoolsAndDepositMustBePositive = 204,
    ValidateFeeBpsTotalFeesCantBeGreaterThan100 = 205,

    GetDepositAmountsMinABiggerThenDesiredA = 206,
    GetDepositAmountsMinBBiggerThenDesiredB = 207,
    GetDepositAmountsAmountABiggerThenDesiredA = 208,
    GetDepositAmountsAmountALessThenMinA = 209,
    GetDepositAmountsAmountBBiggerThenDesiredB = 210,
    GetDepositAmountsAmountBLessThenMinB = 211,
    TotalSharesEqualZero = 212,
    DesiredAmountsBelowOrEqualZero = 213,
    MinAmountsBelowZero = 214,
    AssetNotInPool = 215,
    AlreadyInitialized = 216,
    TokenABiggerThanTokenB = 217,
    InvalidBps = 218,
    SlippageInvalid = 219,

    SwapMinReceivedBiggerThanReturn = 220,
    TransactionAfterTimestampDeadline = 221,
    CannotConvertU256ToI128 = 222,
    UserDeclinesPoolFee = 223,
    SwapFeeBpsOverLimit = 224,
    NotEnoughSharesToBeMinted = 225,
    NotEnoughLiquidityProvided = 226,
    AdminNotSet = 227,
    ContractMathError = 228,
    NegativeInputProvided = 229,
}
