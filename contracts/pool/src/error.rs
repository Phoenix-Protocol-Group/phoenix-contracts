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
    ValidateFeeBpsTotalFeesCantBeGreaterThen100 = 6,

    GetDepositAmountsMinABiggerThenDesiredA = 7,
    GetDepositAmountsMinBBiggerThenDesiredB = 8,
    GetDepositAmountsAmountABiggerThenDesiredA = 9,
    GetDepositAmountsAmountALessThenMinA = 10,
    GetDepositAmountsAmountBBiggerThenDesiredB = 11,
    GetDepositAmountsAmountBLessThenMinB = 12,
}
