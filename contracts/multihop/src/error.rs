use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    AlreadyInitialized = 200,
    OperationsEmpty = 201,
    IncorrectAssetSwap = 202,
    AdminNotSet = 203,
    SameAdmin = 204,
    NoAdminChangeInPlace = 205,
    AdminChangeExpired = 206,
}
