use curve::{Curve, SaturatingLinear};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    vec, Address, Env, String,
};

use crate::{
    storage::{MinterInfo, VestingBalance, VestingTokenInfo},
    tests::setup::instantiate_vesting_client,
};

#[test]
fn transfer_tokens_succesfully() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);
    let vester2 = Address::generate(&env);
    let whitelisted_account = Address::generate(&env);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: Address::generate(&env),
        total_supply: 0,
    };
    let vesting_balances = vec![
        &env,
        VestingBalance {
            address: vester1.clone(),
            balance: 100,
            curve: Curve::SaturatingLinear(SaturatingLinear {
                min_x: 15,
                min_y: 1,
                max_x: 60,
                max_y: 120,
            }),
        },
        VestingBalance {
            address: vester2.clone(),
            balance: 100,
            curve: Curve::SaturatingLinear(SaturatingLinear {
                min_x: 30,
                min_y: 2,
                max_x: 120,
                max_y: 240,
            }),
        },
    ];

    let minter_info = &MinterInfo {
        address: Address::generate(&env),
        cap: Curve::SaturatingLinear(SaturatingLinear {
            min_x: 30,
            min_y: 2,
            max_x: 120,
            max_y: 240,
        }),
    };

    let allowed_vesters = vec![&env, whitelisted_account.clone()];

    let vesting_client = instantiate_vesting_client(&env);
    env.ledger().with_mut(|li| li.timestamp = 1000);
    vesting_client.initialize(
        &admin,
        &vesting_token,
        &vesting_balances,
        minter_info,
        &Some(allowed_vesters),
        &10u32,
    );

    vesting_client.transfer_token(&vester1, &vester2, &100);
}
