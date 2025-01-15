use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    SpreadExceedsLimit = 1,

    ProvideLiquiditySlippageToleranceTooHigh = 2,
    ProvideLiquidityAtLeastOneTokenMustBeBiggerThenZero = 3,

    WithdrawLiquidityMinimumAmountOfAOrBIsNotSatisfied = 4,
    SplitDepositBothPoolsAndDepositMustBePositive = 5,
    ValidateFeeBpsTotalFeesCantBeGreaterThan100 = 6,

    GetDepositAmountsMinABiggerThenDesiredA = 7,
    GetDepositAmountsMinBBiggerThenDesiredB = 8,
    GetDepositAmountsAmountABiggerThenDesiredA = 9,
    GetDepositAmountsAmountALessThenMinA = 10,
    GetDepositAmountsAmountBBiggerThenDesiredB = 11,
    GetDepositAmountsAmountBLessThenMinB = 12,
    TotalSharesEqualZero = 13,
    DesiredAmountsBelowOrEqualZero = 14,
    MinAmountsBelowZero = 15,
    AssetNotInPool = 16,
    TokenABiggerThanTokenB = 17,
    InvalidBps = 18,
    SlippageInvalid = 19,
    SwapMinReceivedBiggerThanReturn = 20,
    TransactionAfterTimestampDeadline = 21,
    CannotConvertU256ToI128 = 22,
    UserDeclinesPoolFee = 23,
    SwapFeeBpsOverLimit = 24,
    NotEnoughSharesToBeMinted = 25,
    NotEnoughLiquidityProvided = 26,
    AdminNotSet = 27,
}
