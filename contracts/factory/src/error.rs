use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    AlreadyInitialized = 1,
    WhiteListeEmpty = 2,
    NotAuthorized = 3,
    LiquidityPoolNotFound = 4,
    TokenABiggerThanTokenB = 5,
    MinStakeInvalid = 6,
    MinRewardInvalid = 7,
    AdminNotSet = 8,
    OverflowingOps = 9,
    SameAdmin = 10,
    NoAdminChangeInPlace = 11,
    AdminChangeExpired = 12,
}
