extern crate std;

use phoenix::utils::AdminChange;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, String,
};

use crate::{
    contract::{Trader, TraderClient},
    error::ContractError,
    storage::PENDING_ADMIN,
};

#[test]
fn propose_admin() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);

    let trader = TraderClient::new(
        &env,
        &env.register(
            Trader,
            (
                &admin,
                String::from_str(&env, "Trader"),
                &(Address::generate(&env), Address::generate(&env)),
                &Address::generate(&env),
            ),
        ),
    );

    let result = trader.propose_admin(&new_admin, &None);
    assert_eq!(result, new_admin.clone());

    let pending_admin: AdminChange = env.as_contract(&trader.address, || {
        env.storage().instance().get(&PENDING_ADMIN).unwrap()
    });

    assert_eq!(trader.query_admin_address(), admin);
    assert_eq!(pending_admin.new_admin, new_admin);
    assert_eq!(pending_admin.time_limit, None);
}

#[test]
fn replace_admin_fails_when_new_admin_is_same_as_current() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    let trader = TraderClient::new(
        &env,
        &env.register(
            Trader,
            (
                &admin,
                String::from_str(&env, "Trader"),
                &(Address::generate(&env), Address::generate(&env)),
                &Address::generate(&env),
            ),
        ),
    );

    assert_eq!(
        trader.try_propose_admin(&admin, &None),
        Err(Ok(ContractError::SameAdmin))
    );
    assert_eq!(trader.query_admin_address(), admin);
}

#[test]
fn accept_admin_successfully() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);

    let trader = TraderClient::new(
        &env,
        &env.register(
            Trader,
            (
                &admin,
                String::from_str(&env, "Trader"),
                &(Address::generate(&env), Address::generate(&env)),
                &Address::generate(&env),
            ),
        ),
    );

    trader.propose_admin(&new_admin, &None);
    assert_eq!(trader.query_admin_address(), admin);

    let result = trader.accept_admin();
    assert_eq!(result, new_admin.clone());
    assert_eq!(trader.query_admin_address(), new_admin);

    let pending_admin: Option<AdminChange> = env.as_contract(&trader.address, || {
        env.storage().instance().get(&PENDING_ADMIN)
    });
    assert!(pending_admin.is_none());
}

#[test]
fn accept_admin_fails_when_no_pending_admin() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    let trader = TraderClient::new(
        &env,
        &env.register(
            Trader,
            (
                &admin,
                String::from_str(&env, "Trader"),
                &(Address::generate(&env), Address::generate(&env)),
                &Address::generate(&env),
            ),
        ),
    );

    assert_eq!(
        trader.try_accept_admin(),
        Err(Ok(ContractError::NoAdminChangeInPlace))
    );

    assert_eq!(trader.query_admin_address(), admin);
}

#[test]
fn accept_admin_fails_when_time_limit_expired() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);

    let trader = TraderClient::new(
        &env,
        &env.register(
            Trader,
            (
                &admin,
                String::from_str(&env, "Trader"),
                &(Address::generate(&env), Address::generate(&env)),
                &Address::generate(&env),
            ),
        ),
    );

    let time_limit = 1000u64;
    trader.propose_admin(&new_admin, &Some(time_limit));
    env.ledger().set_timestamp(time_limit + 100);

    assert_eq!(
        trader.try_accept_admin(),
        Err(Ok(ContractError::AdminChangeExpired))
    );
    assert_eq!(trader.query_admin_address(), admin);
}

#[test]
fn accept_admin_successfully_with_time_limit() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);

    let trader = TraderClient::new(
        &env,
        &env.register(
            Trader,
            (
                &admin,
                String::from_str(&env, "Trader"),
                &(Address::generate(&env), Address::generate(&env)),
                &Address::generate(&env),
            ),
        ),
    );

    let time_limit = 1_500;
    trader.propose_admin(&new_admin, &Some(time_limit));
    assert_eq!(trader.query_admin_address(), admin);

    env.ledger().set_timestamp(1_000u64);

    let result = trader.accept_admin();
    assert_eq!(result, new_admin);
    assert_eq!(trader.query_admin_address(), new_admin);

    let pending_admin: Option<AdminChange> = env.as_contract(&trader.address, || {
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

    let trader_client = TraderClient::new(
        &env,
        &env.register(
            Trader,
            (
                &admin,
                String::from_str(&env, "Trader"),
                &(Address::generate(&env), Address::generate(&env)),
                &Address::generate(&env),
            ),
        ),
    );

    let time_limit = 1_500;
    trader_client.propose_admin(&new_admin, &Some(time_limit));
    assert_eq!(trader_client.query_admin_address(), admin);

    env.ledger().set_timestamp(time_limit);

    let result = trader_client.accept_admin();
    assert_eq!(result, new_admin);
    assert_eq!(trader_client.query_admin_address(), new_admin);

    let pending_admin: Option<AdminChange> = env.as_contract(&trader_client.address, || {
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

    let trader_client = TraderClient::new(
        &env,
        &env.register(
            Trader,
            (
                &admin,
                String::from_str(&env, "Trader"),
                &(Address::generate(&env), Address::generate(&env)),
                &Address::generate(&env),
            ),
        ),
    );

    trader_client.propose_admin(&new_admin, &None);
    trader_client.revoke_admin_change();

    let pending_admin: Option<AdminChange> = env.as_contract(&trader_client.address, || {
        env.storage().instance().get(&PENDING_ADMIN)
    });

    assert!(pending_admin.is_none());
}

#[test]
fn revoke_admin_should_fail_when_no_admin_change_in_place() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    let trader_client = TraderClient::new(
        &env,
        &env.register(
            Trader,
            (
                &admin,
                String::from_str(&env, "Trader"),
                &(Address::generate(&env), Address::generate(&env)),
                &Address::generate(&env),
            ),
        ),
    );

    assert_eq!(
        trader_client.try_revoke_admin_change(),
        Err(Ok(ContractError::NoAdminChangeInPlace))
    );
}
