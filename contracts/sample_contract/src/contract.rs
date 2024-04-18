use soroban_sdk::testutils::arbitrary::std::dbg;
use soroban_sdk::{contract, contractimpl, contractmeta, Env};

use crate::storage::VestingInfo;
use crate::{error::ContractError, storage::VestingBalance};

// Metadata that is added on to the WASM custom section
contractmeta!(key = "Description", val = "Phoenix Protocol Vesting");

#[contract]
pub struct Sample;

pub trait SampleTrait {
    fn initialize(env: Env, vesting_balance: VestingBalance) -> Result<(), ContractError>;
}

#[contractimpl]
impl SampleTrait for Sample {
    fn initialize(env: Env, vesting_balance: VestingBalance) -> Result<(), ContractError> {
        dbg!("Before instance set");
        env.storage().instance().set(
            &vesting_balance.address,
            &VestingInfo {
                balance: vesting_balance.balance,
                curve: vesting_balance.curve.clone(),
            },
        );

        dbg!("Before instance get");
        let instance_result: VestingInfo = env
            .storage()
            .instance()
            .get(&vesting_balance.address)
            .unwrap();

        dbg!(instance_result);

        dbg!("Before persistent set");
        env.storage().persistent().set(
            &vesting_balance.address,
            &VestingInfo {
                balance: vesting_balance.balance,
                curve: vesting_balance.curve.clone(),
            },
        );

        dbg!("Before persistent get");
        let persistent_result: VestingInfo = env
            .storage()
            .persistent()
            .get(&vesting_balance.address)
            .unwrap();

        dbg!(persistent_result);

        env.events()
            .publish(("sample", "initialized at: "), env.ledger().timestamp());

        Ok(())
    }
}
