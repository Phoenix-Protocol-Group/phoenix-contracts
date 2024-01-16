use crate::contract::{Factory, FactoryClient};
use soroban_sdk::{Address, Env, String};

use self::setup::install_multihop_wasm;

mod config;
mod setup;

mod queries;
#[test]
#[should_panic(expected = "Factory: Initialize: initializing contract twice is not allowed")]
fn test_deploy_factory_twice_should_fail() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

        &env,

    let multihop = FactoryClient::new(&env, &env.register_contract(None, Factory {}));
    let multihop_wasm_hash = install_multihop_wasm(&env);

    multihop.initialize(
        &admin,
        &multihop_wasm_hash,
        &vec![&env, random_user.clone()],
    );
    multihop.initialize(&admin, &multihop_wasm_hash, &vec![&env, random_user]);
}
