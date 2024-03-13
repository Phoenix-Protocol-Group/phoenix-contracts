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
}
