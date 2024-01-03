use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Referral {
    /// Address of the referral
    pub address: Address,
    /// fee in bps, later parsed to percentage
    pub fee: i64,
}
