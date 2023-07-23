use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    Unauthorized = 0,
    MinStakeLessOrEqualZero = 1,
    StakeLessThenMinBond = 2,
    TokenPerPowerCannotBeZero = 3,
    ConfigNotSet = 4,
    StakeNotFound = 5,
    FailedToLoadFromStorage = 6,
}
