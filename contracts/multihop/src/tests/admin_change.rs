use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env,
};

use crate::{error::ContractError, storage::PENDING_ADMIN, tests::setup::deploy_multihop_contract};
use phoenix::utils::AdminChange;

#[test]
fn propose_admin() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);

    let multihop = deploy_multihop_contract(&env, admin.clone(), &Address::generate(&env));

    let result = multihop.propose_admin(&new_admin, &None);
    assert_eq!(result, new_admin.clone());

    let pending_admin: AdminChange = env.as_contract(&multihop.address, || {
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

    let multihop = deploy_multihop_contract(&env, admin.clone(), &Address::generate(&env));

    assert_eq!(
        multihop.try_propose_admin(&admin, &None),
        Err(Ok(ContractError::SameAdmin))
    );
}

#[test]
fn accept_admin_successfully() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);

    let multihop = deploy_multihop_contract(&env, admin.clone(), &Address::generate(&env));

    multihop.propose_admin(&new_admin, &None);

    let result = multihop.accept_admin();
    assert_eq!(result, new_admin.clone());

    let pending_admin: Option<AdminChange> = env.as_contract(&multihop.address, || {
        env.storage().instance().get(&PENDING_ADMIN)
    });
    assert!(pending_admin.is_none());
}

#[test]
fn accept_admin_fails_when_no_pending_admin() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    soroban_sdk::testutils::arbitrary::std::dbg!();
    let multihop = deploy_multihop_contract(&env, admin.clone(), &Address::generate(&env));
    soroban_sdk::testutils::arbitrary::std::dbg!();

    assert_eq!(
        multihop.try_accept_admin(),
        Err(Ok(ContractError::NoAdminChangeInPlace))
    )
}

#[test]
fn accept_admin_fails_when_time_limit_expired() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);

    let multihop = deploy_multihop_contract(&env, admin.clone(), &Address::generate(&env));

    let time_limit = 1000u64;
    multihop.propose_admin(&new_admin, &Some(time_limit));
    env.ledger().set_timestamp(time_limit + 100);

    assert_eq!(
        multihop.try_accept_admin(),
        Err(Ok(ContractError::AdminChangeExpired))
    )
}

#[test]
fn accept_admin_successfully_with_time_limit() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);

    let multihop = deploy_multihop_contract(&env, admin.clone(), &Address::generate(&env));

    let time_limit = 1_500;
    multihop.propose_admin(&new_admin, &Some(time_limit));

    env.ledger().set_timestamp(1_000u64);

    let result = multihop.accept_admin();
    assert_eq!(result, new_admin.clone());

    let pending_admin: Option<AdminChange> = env.as_contract(&multihop.address, || {
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

    let multihop = deploy_multihop_contract(&env, admin.clone(), &Address::generate(&env));

    let time_limit = 1_500;
    multihop.propose_admin(&new_admin, &Some(time_limit));

    env.ledger().set_timestamp(time_limit);

    let result = multihop.accept_admin();
    assert_eq!(result, new_admin.clone());

    let pending_admin: Option<AdminChange> = env.as_contract(&multihop.address, || {
        env.storage().instance().get(&PENDING_ADMIN)
    });
    assert!(pending_admin.is_none());
}

#[test]
fn propose_admin_then_revoke() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);

    let multihop = deploy_multihop_contract(&env, admin.clone(), &Address::generate(&env));

    multihop.propose_admin(&new_admin, &None);
    multihop.revoke_admin_change();

    let pending_admin: Option<AdminChange> = env.as_contract(&multihop.address, || {
        env.storage().instance().get(&PENDING_ADMIN)
    });

    assert!(pending_admin.is_none());
}

#[test]
fn revoke_admin_should_fail_when_no_admin_change_in_place() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    let multihop = deploy_multihop_contract(&env, admin.clone(), &Address::generate(&env));

    assert_eq!(
        multihop.try_revoke_admin_change(),
        Err(Ok(ContractError::NoAdminChangeInPlace))
    );
}
