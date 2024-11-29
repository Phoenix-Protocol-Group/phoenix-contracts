use phoenix::ttl::{PERSISTENT_BUMP_AMOUNT, PERSISTENT_LIFETIME_THRESHOLD};
use setup::install_and_deploy_token_contract;
use soroban_sdk::{
    testutils::{storage::Persistent, Address as _, Ledger},
    vec, Address, Env, String,
};

use crate::{
    contract::{self},
    storage::PairTupleKey,
};

use self::setup::{
    deploy_factory_contract, install_lp_contract, install_multihop_wasm, install_stable_lp,
    install_stake_wasm, install_token_wasm,
};

mod config;
mod setup;

mod queries;
#[test]
#[should_panic(expected = "Factory: Initialize: initializing contract twice is not allowed")]
fn test_deploy_factory_twice_should_fail() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);

    let auth_user = Address::generate(&env);
    let multihop_wasm_hash = install_multihop_wasm(&env);
    let lp_wasm_hash = install_lp_contract(&env);
    let stable_wasm_hash = install_stable_lp(&env);
    let stake_wasm_hash = install_stake_wasm(&env);
    let token_wasm_hash = install_token_wasm(&env);

    let factory = deploy_factory_contract(&env, admin.clone());

    factory.initialize(
        &admin,
        &multihop_wasm_hash,
        &lp_wasm_hash,
        &stable_wasm_hash,
        &stake_wasm_hash,
        &token_wasm_hash,
        &vec![&env, auth_user.clone()],
        &10u32,
    );
    factory.initialize(
        &admin,
        &multihop_wasm_hash,
        &lp_wasm_hash,
        &stable_wasm_hash,
        &stake_wasm_hash,
        &token_wasm_hash,
        &vec![&env, auth_user.clone()],
        &10u32,
    );
}

#[test]
fn test_query_for_pool_by_token_pair_ttl_extension() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let token_a = install_and_deploy_token_contract(
        &env,
        &admin.clone(),
        &7,
        &String::from_str(&env, "EURO Coin"),
        &String::from_str(&env, "EURC"),
    );
    let token_b = install_and_deploy_token_contract(
        &env,
        &admin.clone(),
        &7,
        &String::from_str(&env, "USD Coin"),
        &String::from_str(&env, "USDC"),
    );
    let pool_address = Address::generate(&env);

    let client = deploy_factory_contract(&env, admin);
    let contract_id = client.address.clone();

    let key = PairTupleKey {
        token_a: token_a.address.clone(),
        token_b: token_b.address.clone(),
    };

    // the pool address goes the key
    env.as_contract(&contract_id, || {
        env.storage().persistent().set(&key, &pool_address);

        // verifying initial ttl
        let initial_ttl = env.storage().persistent().get_ttl(&key);
        assert!(initial_ttl > 0);
    });

    // calling the function that should extend the ttl
    let result_pool_address =
        client.query_for_pool_by_token_pair(&token_a.address.clone(), &token_b.address.clone());

    assert_eq!(result_pool_address, pool_address);

    // making sure ttl has ben actualy extended
    env.as_contract(&contract_id, || {
        let extended_ttl = env.storage().persistent().get_ttl(&key);
        assert!(extended_ttl > PERSISTENT_LIFETIME_THRESHOLD);
    });

    // move ledger forward in time
    env.ledger().with_mut(|li| {
        li.sequence_number += 10_000;
    });

    // check the ttl after the move
    env.as_contract(&contract_id, || {
        let current_ttl = env.storage().persistent().get_ttl(&key);
        assert!(current_ttl < PERSISTENT_BUMP_AMOUNT);
    });

    // calling the fn again
    let result_pool_address_again =
        client.query_for_pool_by_token_pair(&token_a.address.clone(), &token_b.address.clone());

    assert_eq!(result_pool_address_again, pool_address);

    // again we verify that we have extended
    env.as_contract(&contract_id, || {
        let extended_ttl_after = env.storage().persistent().get_ttl(&key);
        assert!(extended_ttl_after > PERSISTENT_LIFETIME_THRESHOLD);
    });
}
