use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    WhiteListeEmpty = 1,
    NotAuthorized = 2,
    LiquidityPoolNotFound = 3,
    TokenABiggerThanTokenB = 4,
    MinStakeInvalid = 5,
    MinRewardInvalid = 6,
    AdminNotSet = 7,
}
