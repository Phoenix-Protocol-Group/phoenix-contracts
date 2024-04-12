use curve::{Curve, SaturatingLinear};
use soroban_sdk::vec;
use soroban_sdk::{testutils::Address as _, Address, Env};

use soroban_sdk::testutils::arbitrary::std::dbg;

use crate::storage::VestingInfo;
use crate::{
    contract::{Sample, SampleClient},
    storage::VestingBalance,
};

#[test]
fn test_get_from_storage() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();
    let user = Address::generate(&env);

    let sample_client = SampleClient::new(&env, &env.register_contract(None, Sample {}));
    let vesting_balances = vec![
        &env,
        VestingBalance {
            address: user.clone(),
            balance: 200,
            curve: Curve::SaturatingLinear(SaturatingLinear {
                min_x: 15,
                min_y: 120,
                max_x: 60,
                max_y: 0,
            }),
        },
    ];

    sample_client.initialize();

    vesting_balances.clone().into_iter().for_each(|vb| {
        sample_client.save_vesting_in_instance(
            &vb.address,
            &VestingInfo {
                amount: vb.balance,
                curve: vb.curve,
            },
        );
    });

    let result_instance = sample_client.query_vesting_in_instance(&user);

    dbg!(result_instance);

    vesting_balances.into_iter().for_each(|vb| {
        sample_client.save_vesting_in_persistent(
            &vb.address,
            &VestingInfo {
                amount: vb.balance,
                curve: vb.curve,
            },
        );
    });

    let result_persistent = sample_client.query_vesting_in_persistent(&user);

    dbg!(result_persistent);
}
