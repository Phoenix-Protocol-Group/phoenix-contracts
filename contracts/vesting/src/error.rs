use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    VestingScheduleNotFoundForAddress = 1,
    AllowanceNotFoundForGivenPair = 2,
}
