use curve::CurveError;
use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    VestingNotFoundForAddress = 700,
    AllowanceNotFoundForGivenPair = 701,
    MinterNotFound = 702,
    NoBalanceFoundForAddress = 703,
    NoConfigFound = 704,
    NoAdminFound = 705,
    MissingBalance = 706,
    VestingComplexityTooHigh = 707,
    TotalVestedOverCapacity = 708,
    InvalidTransferAmount = 709,
    CantMoveVestingTokens = 710,
    NotEnoughCapacity = 711,
    NotAuthorized = 712,
    NeverFullyVested = 713,
    VestsMoreThanSent = 714,
    InvalidBurnAmount = 715,
    InvalidMintAmount = 716,
    InvalidAllowanceAmount = 717,
    DuplicateInitialBalanceAddresses = 718,
    CurveError = 719,
    NoWhitelistFound = 720,
    NoTokenInfoFound = 721,
    NoVestingComplexityValueFound = 722,
    NoAddressesToAdd = 723,
    NoEnoughtTokensToStart = 724,
    NotEnoughBalance = 725,

    VestingBothPresent = 726,
    VestingNonePresent = 727,

    CurveConstant = 728,
    CurveSLNotDecreasing = 729,
    AlreadyInitialized = 730,
    AdminNotFound = 731,
    ContractMathError = 732,

    SameAdmin = 733,
    NoAdminChangeInPlace = 734,
    AdminChangeExpired = 735,
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
