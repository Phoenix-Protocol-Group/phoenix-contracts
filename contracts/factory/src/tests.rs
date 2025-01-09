use soroban_sdk::{testutils::Address as _, vec, Address, Env};

use crate::contract::{Factory, FactoryClient};

use self::setup::{
    install_lp_contract, install_multihop_wasm, install_stable_lp, install_stake_wasm,
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
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);

    let auth_user = Address::generate(&env);
    let multihop_wasm_hash = install_multihop_wasm(&env);
    let lp_wasm_hash = install_lp_contract(&env);
    let stable_wasm_hash = install_stable_lp(&env);
    let stake_wasm_hash = install_stake_wasm(&env);
    let token_wasm_hash = install_token_wasm(&env);
    let whitelisted_accounts = vec![&env, auth_user];
    let contract_addr = Address::generate(&env);

    let _ = FactoryClient::new(
        &env,
        &env.register_at(
            &contract_addr.clone(),
            Factory {},
            (
                &admin,
                &multihop_wasm_hash,
                &lp_wasm_hash,
                &stable_wasm_hash,
                &stake_wasm_hash,
                &token_wasm_hash,
                whitelisted_accounts.clone(),
                &10u32,
            ),
        ),
    );

    let _ = FactoryClient::new(
        &env,
        &env.register_at(
            &contract_addr,
            Factory {},
            (
                &admin,
                &multihop_wasm_hash,
                &lp_wasm_hash,
                &stable_wasm_hash,
                &stake_wasm_hash,
                &token_wasm_hash,
                whitelisted_accounts,
                &10u32,
            ),
        ),
    );
}
