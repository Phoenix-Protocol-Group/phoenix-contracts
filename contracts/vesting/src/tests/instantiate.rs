use soroban_sdk::{testutils::Address as _, vec, Address, Env, String};

use crate::{
    storage::{MinterInfo, VestingInfo, VestingSchedule, VestingTokenInfo},
    tests::setup::{deploy_token_contract, instantiate_vesting_client},
};
use curve::{Curve, SaturatingLinear};

#[test]
fn instantiate_contract_successfully() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);
    let vester2 = Address::generate(&env);

    let token_client = deploy_token_contract(&env, &admin);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: token_client.address.clone(),
    };
    let vesting_schedules = vec![
        &env,
        VestingSchedule {
            recipient: vester1.clone(),
            curve: Curve::SaturatingLinear(SaturatingLinear {
                min_x: 15,
                min_y: 120,
                max_x: 60,
                max_y: 0,
            }),
        },
        VestingSchedule {
            recipient: vester2,
            curve: Curve::SaturatingLinear(SaturatingLinear {
                min_x: 30,
                min_y: 240,
                max_x: 120,
                max_y: 0,
            }),
        },
    ];

    let vesting_client = instantiate_vesting_client(
        &env,
        &admin,
        vesting_token.clone(),
        10u32,
        None::<MinterInfo>,
    );

    token_client.mint(&admin, &480);
    vesting_client.create_vesting_schedules(&vesting_schedules);

    assert_eq!(vesting_client.query_token_info(), vesting_token);
    assert_eq!(
        vesting_client.query_all_vesting_info(&vester1),
        vec![
            &env,
            VestingInfo {
                recipient: vester1,
                balance: 120,
                schedule: Curve::SaturatingLinear(SaturatingLinear {
                    min_x: 15,
                    min_y: 120,
                    max_x: 60,
                    max_y: 0,
                })
            }
        ]
    );
}

#[should_panic(
    expected = "Vesting: Create vesting account: At least one vesting schedule must be provided."
)]
#[test]
fn instantiate_contract_without_any_vesting_balances_should_fail() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_client = deploy_token_contract(&env, &admin);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: token_client.address.clone(),
    };
    let vesting_schedules = vec![&env];

    let vesting_client = instantiate_vesting_client(
        &env,
        &admin,
        vesting_token.clone(),
        10u32,
        None::<MinterInfo>,
    );

    token_client.mint(&admin, &100);
    vesting_client.create_vesting_schedules(&vesting_schedules);
}

#[should_panic(
    expected = "Vesting: Create vesting account: Admin does not have enough tokens to start the vesting schedule"
)]
#[test]
fn create_schedule_panics_when_admin_has_no_tokens_to_fund() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);

    let token_client = deploy_token_contract(&env, &admin);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: token_client.address.clone(),
    };
    let vesting_schedules = vec![
        &env,
        VestingSchedule {
            recipient: vester1,
            curve: Curve::SaturatingLinear(SaturatingLinear {
                min_x: 15,
                min_y: 120,
                max_x: 60,
                max_y: 0,
            }),
        },
    ];

    let vesting_client = instantiate_vesting_client(
        &env,
        &admin,
        vesting_token.clone(),
        10u32,
        None::<MinterInfo>,
    );

    vesting_client.create_vesting_schedules(&vesting_schedules);
}
