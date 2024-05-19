use soroban_sdk::{testutils::Address as _, vec, Address, Env, String};

use crate::{
    storage::{DistributionInfo, VestingSchedule, VestingTokenInfo},
    tests::setup::{deploy_token_contract, instantiate_vesting_client},
};

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
            distribution_info: DistributionInfo {
                start_timestamp: 15,
                end_timestamp: 60,
                amount: 120,
            },
        },
        VestingSchedule {
            recipient: vester2,
            distribution_info: DistributionInfo {
                start_timestamp: 30,
                end_timestamp: 120,
                amount: 240,
            },
        },
    ];

    let vesting_client = instantiate_vesting_client(&env);
    token_client.mint(&admin, &480);
    vesting_client.initialize(&admin, &vesting_token, &10u32);
    vesting_client.create_vesting_schedules(&vesting_schedules);

    assert_eq!(vesting_client.query_token_info(), vesting_token);
    assert_eq!(
        vesting_client.query_distribution_info(&vester1),
        vesting_schedules.get(0).unwrap().distribution_info
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

    let vesting_client = instantiate_vesting_client(&env);

    token_client.mint(&admin, &100);
    vesting_client.initialize(&admin, &vesting_token, &10u32);
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
            distribution_info: DistributionInfo {
                start_timestamp: 15,
                end_timestamp: 60,
                amount: 120,
            },
        },
    ];

    let vesting_client = instantiate_vesting_client(&env);

    vesting_client.initialize(&admin, &vesting_token, &10u32);
    vesting_client.create_vesting_schedules(&vesting_schedules);
}
