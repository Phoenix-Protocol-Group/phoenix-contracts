use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    AdminNotFound = 1,
    ContractIdNotFound = 2,
    PairNotFound = 3,
    OutputTokenNotFound = 4,
    MaxSpreadNotFound = 5,
    Unauthorized = 6,
    SwapTokenNotInPair = 7,
}
