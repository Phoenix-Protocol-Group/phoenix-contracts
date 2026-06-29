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

    DelegateNotSet = 333,
    DelegateUnauthorizedToken = 334,
    DelegatedOutUnderflow = 335,
    DelegateInvalidAmount = 336,

    /// `swap` invoked while either pool reserve is below its admin-set
    /// bootstrap floor. The pool is still in deposit-only mode;
    /// `provide_liquidity` and `withdraw_liquidity` are unaffected.
    TradingFloorNotMet = 337,

    /// `provide_liquidity(auto_stake=true)` or
    /// `withdraw_liquidity(auto_unstake=Some(_))` called on a pool that
    /// was deployed without a stake contract (default for `pool_blended`).
    /// The LP share token is still minted/burned normally; the caller
    /// must just pass `auto_stake=false` and skip the unstake hint.
    StakingDisabled = 338,
}
