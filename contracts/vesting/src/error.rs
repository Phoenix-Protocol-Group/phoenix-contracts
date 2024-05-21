use curve::CurveError;
use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    Std = 0,
    VestingNotFoundForAddress = 1,
    AllowanceNotFoundForGivenPair = 2,
    MinterNotFound = 3,
    NoBalanceFoundForAddress = 4,
    NoConfigFound = 5,
    NoAdminFound = 6,
    MissingBalance = 7,
    VestingComplexityTooHigh = 8,
    TotalVestedOverCapacity = 9,
    InvalidTransferAmount = 10,
    CantMoveVestingTokens = 11,
    NotEnoughCapacity = 12,
    NotAuthorized = 13,
    NeverFullyVested = 14,
    VestsMoreThanSent = 15,
    InvalidBurnAmount = 16,
    InvalidMintAmount = 17,
    InvalidAllowanceAmount = 18,
    DuplicateInitialBalanceAddresses = 19,
    CurveError = 20,
    NoWhitelistFound = 21,
    NoTokenInfoFound = 22,
    NoVestingComplexityValueFound = 23,
    NoAddressesToAdd = 24,
    NoEnoughtTokensToStart = 25,
    NotEnoughBalance = 26,

    VestingBothPresent = 27,
    VestingNonePresent = 28,

    CurveConstant = 29,
    CurveSLNotDecreasing = 30,
}

impl From<CurveError> for ContractError {
    fn from(_: CurveError) -> Self {
        ContractError::CurveError
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_from_curve_error() {
        let curve_error = CurveError::TooComplex;
        let contract_error = ContractError::from(curve_error);
        assert_eq!(contract_error, ContractError::CurveError);
    }
}
