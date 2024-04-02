use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    VestingScheduleNotFoundForAddress = 1,
    AllowanceNotFoundForGivenPair = 2,
    MinterNotFoundForAddress = 3,
    NoBalanceFoundForAddress = 4,
    NoConfigFound = 5,
    NoAdminFound = 6,
    MissingBalance = 7,
}
