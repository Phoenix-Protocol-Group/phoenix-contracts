extern crate std;

use phoenix::utils::AdminChange;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, String,
};

use crate::{
    error::ContractError,
    storage::{VestingTokenInfo, PENDING_ADMIN},
    tests::setup::instantiate_vesting_client,
};

#[test]
fn propose_admin() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);

    let vesting_token_info = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: Address::generate(&env),
    };
    let vesting = instantiate_vesting_client(&env);
    vesting.initialize(&admin, &vesting_token_info, &10u32);

    let result = vesting.propose_admin(&new_admin, &None);
    assert_eq!(result, new_admin.clone());

    let pending_admin: AdminChange = env.as_contract(&vesting.address, || {
        env.storage().instance().get(&PENDING_ADMIN).unwrap()
    });

    assert_eq!(pending_admin.new_admin, new_admin);
    assert_eq!(pending_admin.time_limit, None);
}

#[test]
fn replace_admin_fails_when_new_admin_is_same_as_current() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    let vesting_token_info = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: Address::generate(&env),
    };
    let vesting = instantiate_vesting_client(&env);
    vesting.initialize(&admin, &vesting_token_info, &10u32);

    assert_eq!(
        vesting.try_propose_admin(&admin, &None),
        Err(Ok(ContractError::SameAdmin))
    );
}

#[test]
fn accept_admin_successfully() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);

    let vesting_token_info = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: Address::generate(&env),
    };
    let vesting = instantiate_vesting_client(&env);
    vesting.initialize(&admin, &vesting_token_info, &10u32);

    vesting.propose_admin(&new_admin, &None);

    let result = vesting.accept_admin();
    assert_eq!(result, new_admin.clone());

    let pending_admin: Option<AdminChange> = env.as_contract(&vesting.address, || {
        env.storage().instance().get(&PENDING_ADMIN)
    });
    assert!(pending_admin.is_none());
}

#[test]
fn accept_admin_fails_when_no_pending_admin() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    let vesting_token_info = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: Address::generate(&env),
    };
    let vesting = instantiate_vesting_client(&env);
    vesting.initialize(&admin, &vesting_token_info, &10u32);

    assert_eq!(
        vesting.try_accept_admin(),
        Err(Ok(ContractError::NoAdminChangeInPlace))
    );
}

#[test]
fn accept_admin_fails_when_time_limit_expired() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);

    let vesting_token_info = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: Address::generate(&env),
    };
    let vesting = instantiate_vesting_client(&env);
    vesting.initialize(&admin, &vesting_token_info, &10u32);

    let time_limit = 1000u64;
    vesting.propose_admin(&new_admin, &Some(time_limit));
    env.ledger().set_timestamp(time_limit + 100);

    assert_eq!(
        vesting.try_accept_admin(),
        Err(Ok(ContractError::AdminChangeExpired))
    );
}

#[test]
fn accept_admin_successfully_with_time_limit() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);

    let vesting_token_info = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: Address::generate(&env),
    };
    let vesting = instantiate_vesting_client(&env);
    vesting.initialize(&admin, &vesting_token_info, &10u32);

    let time_limit = 1_500;
    vesting.propose_admin(&new_admin, &Some(time_limit));

    env.ledger().set_timestamp(1_000u64);

    let result = vesting.accept_admin();
    assert_eq!(result, new_admin);

    let pending_admin: Option<AdminChange> = env.as_contract(&vesting.address, || {
        env.storage().instance().get(&PENDING_ADMIN)
    });
    assert!(pending_admin.is_none());
}

#[test]
fn accept_admin_successfully_on_time_limit() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);

    let vesting_token_info = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: Address::generate(&env),
    };
    let vesting = instantiate_vesting_client(&env);
    vesting.initialize(&admin, &vesting_token_info, &10u32);

    let time_limit = 1_500;
    vesting.propose_admin(&new_admin, &Some(time_limit));

    env.ledger().set_timestamp(time_limit);

    let result = vesting.accept_admin();
    assert_eq!(result, new_admin);

    let pending_admin: Option<AdminChange> = env.as_contract(&vesting.address, || {
        env.storage().instance().get(&PENDING_ADMIN)
    });
    assert!(pending_admin.is_none());
}
