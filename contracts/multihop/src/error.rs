use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    AlreadyInitialized = 100,
    OperationsEmpty = 101,
    IncorrectAssetSwap = 102,
    AdminNotSet = 103,
}
