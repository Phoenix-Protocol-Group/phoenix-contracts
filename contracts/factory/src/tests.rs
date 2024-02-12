use soroban_sdk::{testutils::Address as _, vec, Address, Env};

use self::setup::{
    deploy_factory_contract, install_lp_contract, install_multihop_wasm, install_stake_wasm,
    install_token_wasm,
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
    let stake_wasm_hash = install_stake_wasm(&env);
    let token_wasm_hash = install_token_wasm(&env);

    let factory = deploy_factory_contract(&env, admin.clone());

    factory.initialize(
        &admin,
        &multihop_wasm_hash,
        &lp_wasm_hash,
        &stake_wasm_hash,
        &token_wasm_hash,
        &vec![&env, auth_user.clone()],
        &10u32,
    );
    factory.initialize(
        &admin,
        &multihop_wasm_hash,
        &lp_wasm_hash,
        &stake_wasm_hash,
        &token_wasm_hash,
        &vec![&env, auth_user.clone()],
        &10u32,
    );
}
