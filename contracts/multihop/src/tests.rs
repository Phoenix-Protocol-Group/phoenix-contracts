use crate::contract::{Multihop, MultihopClient};
use soroban_sdk::{testutils::Address as _, Address, Env};

mod query;
mod setup;
mod swap;

#[test]
#[should_panic(expected = "Multihop: Initialize: initializing contract twice is not allowed")]
fn test_deploy_multihop_twice_should_fail() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let multihop_addr = Address::generate(&env);
    let factory = Address::generate(&env);

    let _ = MultihopClient::new(
        &env,
        &env.register_at(&multihop_addr, Multihop, (&admin, factory.clone())),
    );
    let _ = MultihopClient::new(
        &env,
        &env.register_at(&multihop_addr, Multihop, (admin, factory)),
    );
}
