use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    OperationsEmpty = 1,
    IncorrectAssetSwap = 2,
    AdminNotSet = 3,
}
