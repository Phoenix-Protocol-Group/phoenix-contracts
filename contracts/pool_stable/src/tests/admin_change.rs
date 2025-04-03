extern crate std;

use phoenix::utils::AdminChange;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env,
};

use crate::{
    error::ContractError,
    storage::{DataKey, ADMIN, PENDING_ADMIN},
    tests::setup::{deploy_stable_liquidity_pool_contract, deploy_token_contract},
};

#[test]
fn propose_admin() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);

    let mut token1 = deploy_token_contract(&env, &admin);
    let mut token2 = deploy_token_contract(&env, &admin);

    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
    }

    let pool = deploy_stable_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        0i64,
        None,
        None,
        None,
        Address::generate(&env),
        Address::generate(&env),
        None,
    );

    let result = pool.propose_admin(&new_admin, &None);
    assert_eq!(result, new_admin.clone());

    let pending_admin: AdminChange = env.as_contract(&pool.address, || {
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

    let mut token1 = deploy_token_contract(&env, &admin);
    let mut token2 = deploy_token_contract(&env, &admin);

    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
    }

    let pool = deploy_stable_liquidity_pool_contract(
        &env,
        Some(admin.clone()),
        (&token1.address, &token2.address),
        0i64,
        None,
        None,
        None,
        Address::generate(&env),
        Address::generate(&env),
        None,
    );

    assert_eq!(
        pool.try_propose_admin(&admin, &None),
        Err(Ok(ContractError::SameAdmin))
    );
}

#[test]
fn accept_admin_successfully() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);

    let mut token1 = deploy_token_contract(&env, &admin);
    let mut token2 = deploy_token_contract(&env, &admin);

    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
    }

    let pool = deploy_stable_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        0i64,
        None,
        None,
        None,
        Address::generate(&env),
        Address::generate(&env),
        None,
    );

    pool.propose_admin(&new_admin, &None);

    let result = pool.accept_admin();
    assert_eq!(result, new_admin.clone());

    let pending_admin: Option<AdminChange> = env.as_contract(&pool.address, || {
        env.storage().instance().get(&PENDING_ADMIN)
    });
    assert!(pending_admin.is_none());
}

#[test]
fn accept_admin_fails_when_no_pending_admin() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    let mut token1 = deploy_token_contract(&env, &admin);
    let mut token2 = deploy_token_contract(&env, &admin);

    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
    }

    let pool = deploy_stable_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        0i64,
        None,
        None,
        None,
        Address::generate(&env),
        Address::generate(&env),
        None,
    );

    assert_eq!(
        pool.try_accept_admin(),
        Err(Ok(ContractError::NoAdminChangeInPlace))
    )
}

#[test]
fn accept_admin_fails_when_time_limit_expired() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);

    let mut token1 = deploy_token_contract(&env, &admin);
    let mut token2 = deploy_token_contract(&env, &admin);

    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
    }

    let pool = deploy_stable_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        0i64,
        None,
        None,
        None,
        Address::generate(&env),
        Address::generate(&env),
        None,
    );

    let time_limit = 1000u64;
    pool.propose_admin(&new_admin, &Some(time_limit));
    env.ledger().set_timestamp(time_limit + 100);

    assert_eq!(
        pool.try_accept_admin(),
        Err(Ok(ContractError::AdminChangeExpired))
    )
}

#[test]
fn accept_admin_successfully_with_time_limit() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);

    let mut token1 = deploy_token_contract(&env, &admin);
    let mut token2 = deploy_token_contract(&env, &admin);

    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
    }

    let pool = deploy_stable_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        0i64,
        None,
        None,
        None,
        Address::generate(&env),
        Address::generate(&env),
        None,
    );

    let time_limit = 1_500;
    pool.propose_admin(&new_admin, &Some(time_limit));

    env.ledger().set_timestamp(1_000u64);

    let result = pool.accept_admin();
    assert_eq!(result, new_admin.clone());

    let pending_admin: Option<AdminChange> = env.as_contract(&pool.address, || {
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

    let mut token1 = deploy_token_contract(&env, &admin);
    let mut token2 = deploy_token_contract(&env, &admin);

    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
    }

    let pool = deploy_stable_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        0i64,
        None,
        None,
        None,
        Address::generate(&env),
        Address::generate(&env),
        None,
    );

    let time_limit = 1_500;
    pool.propose_admin(&new_admin, &Some(time_limit));

    env.ledger().set_timestamp(time_limit);

    let result = pool.accept_admin();
    assert_eq!(result, new_admin.clone());

    let pending_admin: Option<AdminChange> = env.as_contract(&pool.address, || {
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

    let mut token1 = deploy_token_contract(&env, &admin);
    let mut token2 = deploy_token_contract(&env, &admin);
    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
    }

    let pool = deploy_stable_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        0i64,
        None,
        None,
        None,
        Address::generate(&env),
        Address::generate(&env),
        None,
    );

    pool.propose_admin(&new_admin, &None);
    pool.revoke_admin_change();

    let pending_admin: Option<AdminChange> = env.as_contract(&pool.address, || {
        env.storage().instance().get(&PENDING_ADMIN)
    });

    assert!(pending_admin.is_none());
}

#[test]
fn revoke_admin_should_fail_when_no_admin_change_in_place() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    let mut token1 = deploy_token_contract(&env, &admin);
    let mut token2 = deploy_token_contract(&env, &admin);
    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
    }

    let pool = deploy_stable_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        0i64,
        None,
        None,
        None,
        Address::generate(&env),
        Address::generate(&env),
        None,
    );

    assert_eq!(
        pool.try_revoke_admin_change(),
        Err(Ok(ContractError::NoAdminChangeInPlace))
    );
}

#[test]
fn migrate_admin_key() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    let mut token1 = deploy_token_contract(&env, &admin);
    let mut token2 = deploy_token_contract(&env, &admin);
    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
    }

    let pool = deploy_stable_liquidity_pool_contract(
        &env,
        None,
        (&token1.address, &token2.address),
        0i64,
        None,
        None,
        None,
        Address::generate(&env),
        Address::generate(&env),
        None,
    );

    let before_migration: Address = env.as_contract(&pool.address, || {
        env.storage().instance().get(&DataKey::Admin).unwrap()
    });

    pool.migrate_admin_key();

    let after_migration: Address = env.as_contract(&pool.address, || {
        env.storage().instance().get(&ADMIN).unwrap()
    });

    assert_eq!(before_migration, after_migration);
    assert_ne!(Address::generate(&env), after_migration)
}
