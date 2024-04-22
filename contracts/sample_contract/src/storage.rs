use curve::Curve;
use soroban_sdk::{contracttype, Address, Env, Vec};

use crate::error::ContractError;
use soroban_sdk::testutils::arbitrary::std::dbg;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VestingBalance {
    pub address: Address,
    pub balance: i128,
    pub curve: Curve,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VestingInfo {
    pub balance: i128,
    pub curve: Curve,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Key {
    pub balance: i128,
    pub curve: Curve,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Value {
    pub address: Address,
    pub balance: i128,
    pub curve: Curve,
}

pub fn create_vesting_accounts(
    env: &Env,
    vesting_balances: Vec<VestingBalance>,
) -> Result<(), ContractError> {
    vesting_balances.into_iter().for_each(|vb| {
        // dbg!("Before instance set");
        // env.storage().instance().set(
        //     &vb.address,
        //     &VestingInfo {
        //         balance: vb.balance,
        //         curve: vb.curve.clone(),
        //     },
        // );
        // dbg!("Before instance get");
        // let instance_result = env
        //     .storage()
        //     .instance()
        //     .get::<Address, VestingInfo>(&vb.address)
        //     .unwrap();

        // dbg!(instance_result);

        dbg!("Before persistent set");
        env.storage().persistent().set::<Address, VestingInfo>(
            &vb.address,
            &VestingInfo {
                balance: vb.balance,
                curve: vb.curve.clone(),
            },
        );

        dbg!("Before persistent get");
        let persistent_result: VestingInfo = env.storage().persistent().get(&vb.address).unwrap();

        dbg!(persistent_result);
    });

    Ok(())
}
