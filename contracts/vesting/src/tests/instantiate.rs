use soroban_sdk::{testutils::Address as _, vec, Address, Env, String};

use crate::{
    contract::{Vesting, VestingClient},
    storage::{MinterInfo, VestingInfoResponse, VestingSchedule, VestingTokenInfo},
    tests::setup::deploy_token_contract,
};
use curve::{Curve, SaturatingLinear};

use super::setup::{install_latest_vesting, old_vesting};

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

    let vesting_client = VestingClient::new(
        &env,
        &env.register(
            Vesting,
            (&admin, vesting_token.clone(), &10u32, None::<MinterInfo>),
        ),
    );

    token_client.mint(&admin, &480);
    vesting_client.create_vesting_schedules(&vesting_schedules);

    assert_eq!(vesting_client.query_token_info(), vesting_token);
    assert_eq!(
        vesting_client.query_all_vesting_info(&vester1),
        vec![
            &env,
            VestingInfoResponse {
                recipient: vester1,
                balance: 120,
                schedule: Curve::SaturatingLinear(SaturatingLinear {
                    min_x: 15,
                    min_y: 120,
                    max_x: 60,
                    max_y: 0,
                }),
                index: 0,
            }
        ]
    );

    let config = vesting_client.query_config();
    assert!(!config.is_with_minter);

    let new_vesting_token = deploy_token_contract(&env, &Address::generate(&env));

    vesting_client.update_vesting_token(&new_vesting_token.address);
    vesting_client.update_max_complexity(&10);
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

    let vesting_client = VestingClient::new(
        &env,
        &env.register(Vesting, (&admin, vesting_token, &10u32, None::<MinterInfo>)),
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

    let vesting_client = VestingClient::new(
        &env,
        &env.register(Vesting, (&admin, vesting_token, &10u32, None::<MinterInfo>)),
    );

    vesting_client.create_vesting_schedules(&vesting_schedules);
}

#[test]
fn test_update_vesting() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    let token_client = deploy_token_contract(&env, &admin);

    let vesting_token = old_vesting::VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: token_client.address.clone(),
    };

    let vesting_addr = env.register(old_vesting::WASM, ());
    let old_vesting = old_vesting::Client::new(&env, &vesting_addr);

    old_vesting.initialize(&admin, &vesting_token, &6);

    let new_wasm_hash = install_latest_vesting(&env);
    old_vesting.update(&new_wasm_hash);

    let latest_vesting = VestingClient::new(&env, &old_vesting.address);
    assert_eq!(
        latest_vesting.query_token_info().address,
        vesting_token.address
    );
}

#[test]
fn test_update_vesting_with_minter() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    let token_client = deploy_token_contract(&env, &admin);

    let vesting_token = old_vesting::VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: token_client.address.clone(),
    };

    let vesting_addr = env.register(old_vesting::WASM, ());
    let old_vesting = old_vesting::Client::new(&env, &vesting_addr);

    let minter_addr = Address::generate(&env);
    let minter_capacity = 6;
    let minter_info = old_vesting::MinterInfo {
        address: minter_addr.clone(),
        mint_capacity: minter_capacity,
    };

    old_vesting.initialize_with_minter(&admin, &vesting_token, &6, &minter_info);

    let new_wasm_hash = install_latest_vesting(&env);
    old_vesting.update(&new_wasm_hash);

    let latest_vesting = VestingClient::new(&env, &old_vesting.address);
    assert_eq!(latest_vesting.query_minter().address, minter_addr);
}
