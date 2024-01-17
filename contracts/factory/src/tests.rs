use crate::contract::{Factory, FactoryClient};
use soroban_sdk::{testutils::Address as _, vec, Address, Env};

use self::setup::{
    install_lp_contract, install_multihop_wasm, install_stake_wasm, install_token_wasm,
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
    let multihop = FactoryClient::new(&env, &env.register_contract(None, Factory {}));
    let multihop_wasm_hash = install_multihop_wasm(&env);
    let lp_wasm_hash = install_lp_contract(&env);
    let stake_wasm_hash = install_stake_wasm(&env);
    let token_wasm_hash = install_token_wasm(&env);

    factory.initialize(
        &admin,
        &multihop_wasm_hash,
        &lp_wasm_hash,
        &stake_wasm_hash,
        &token_wasm_hash,
        &vec![&env, auth_user.clone()],
    );
    factory.initialize(
        &admin,
        &multihop_wasm_hash,
        &lp_wasm_hash,
        &stake_wasm_hash,
        &token_wasm_hash,
        &vec![&env, auth_user.clone()],
    );
}
