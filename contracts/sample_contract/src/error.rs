use curve::CurveError;
use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    Std = 0,
    VestingNotFoundForAddress = 1,
    SupplyOverTheCap = 2,
}

impl From<CurveError> for ContractError {
    fn from(_: CurveError) -> Self {
        ContractError::Std
    }
}
