use soroban_sdk::testutils::arbitrary::std::dbg;
use soroban_sdk::{contract, contractimpl, contractmeta, Address, Env, Vec};

use crate::storage::{create_vesting_accounts, VestingInfo};
use crate::{error::ContractError, storage::VestingBalance};

// Metadata that is added on to the WASM custom section
contractmeta!(key = "Description", val = "Phoenix Protocol Vesting");

#[contract]
pub struct Sample;

pub trait SampleTrait {
    fn initialize(env: Env, vesting_balances: Vec<VestingBalance>) -> Result<(), ContractError>;

    fn query(env: &Env, address: Address) -> Result<VestingInfo, ContractError>;
}

#[contractimpl]
impl SampleTrait for Sample {
    fn initialize(env: Env, vesting_balances: Vec<VestingBalance>) -> Result<(), ContractError> {
        create_vesting_accounts(&env, vesting_balances)?;

        env.events()
            .publish(("sample", "initialized at: "), env.ledger().timestamp());

        Ok(())
    }

    fn query(env: &Env, address: Address) -> Result<VestingInfo, ContractError> {
        let result = env.storage().persistent().get(&address);

        match result {
            Some(vi) => Ok(vi),
            None => Err(ContractError::Std),
        }
    }
}
