use crate::contract::{Factory, FactoryClient};
use soroban_sdk::{testutils::Address as _, Address, Env};

mod config;
mod setup;

mod queries;
#[test]
#[should_panic(expected = "Factory: Initialize: initializing contract twice is not allowed")]
fn test_deploy_factory_twice_should_fail() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::random(&env);

    let multihop = FactoryClient::new(&env, &env.register_contract(None, Factory {}));
    multihop.initialize(&admin);
    multihop.initialize(&admin);
}
