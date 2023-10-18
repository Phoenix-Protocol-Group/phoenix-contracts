use crate::contract::{Multihop, MultihopClient};
use crate::tests::setup::deploy_factory_contract;
use soroban_sdk::{testutils::Address as _, Address, Env};

mod query;
mod setup;
mod swap;

#[test]
#[should_panic(expected = "Multihop: Initialize: initializing contract twice is not allowed")]
fn test_deploy_multihop_twice_should_fail() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::random(&env);

    let multihop = MultihopClient::new(&env, &env.register_contract(None, Multihop {}));
    let factory = deploy_factory_contract(&env, admin.clone());
    multihop.initialize(&admin, &factory);
    multihop.initialize(&admin, &factory);
}
