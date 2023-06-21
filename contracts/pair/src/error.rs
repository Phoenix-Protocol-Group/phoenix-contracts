use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    FirstTokenMustBeSmallerThenSecond = 1,
    SlippageToleranceExceeded = 2,
    SlippageToleranceViolated = 3,
    SpreadExceedsMaxAllowed = 4,
}
