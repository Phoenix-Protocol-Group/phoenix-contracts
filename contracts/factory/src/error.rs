use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    AlreadyInitialized = 100,
    WhiteListeEmpty = 101,
    NotAuthorized = 102,
    LiquidityPoolNotFound = 103,
    TokenABiggerThanTokenB = 104,
    MinStakeInvalid = 105,
    MinRewardInvalid = 106,
    AdminNotSet = 107,
    OverflowingOps = 108,
    SameAdmin = 109,
    NoAdminChangeInPlace = 110,
    AdminChangeExpired = 111,
}
