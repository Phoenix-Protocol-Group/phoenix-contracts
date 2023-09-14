use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    Unauthorized = 0,
    ConfigNotSet = 1,
    FailedToGetAdminAddrFromStorage = 2,
    FirstTokenMustBeSmallerThenSecond = 3,
    LiquidityPoolVectorNotFound = 4,
    MinStakeLessOrEqualZero = 5,
    MinRewardTooSmall = 6,
    ContractNotDeployed = 7,
    LiquidityPoolPairNotFound = 8,
}
