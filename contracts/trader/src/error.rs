use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    AdminNotFound = 601,
    ContractIdNotFound = 602,
    PairNotFound = 603,
    OutputTokenNotFound = 604,
    MaxSpreadNotFound = 605,
    Unauthorized = 606,
    SwapTokenNotInPair = 607,
    InvalidMaxSpreadBps = 608,
    InitValueNotFound = 609,
    AlreadyInitialized = 610,
    AdminNotSet = 611,
    SameAdmin = 612,
    NoAdminChangeInPlace = 613,
    AdminChangeExpired = 614,
}
