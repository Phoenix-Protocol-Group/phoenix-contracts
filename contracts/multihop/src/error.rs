use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    AlreadyInitialized = 1,
    OperationsEmpty = 2,
    IncorrectAssetSwap = 3,
    AdminNotSet = 4,
}
