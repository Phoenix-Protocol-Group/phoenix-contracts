use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    Unauthorized = 0,
    AdminNotFound = 1,
    FactoryNotFound = 2,
    RemoteCallFailed = 555,
    OperationsEmpty = 4,
}
