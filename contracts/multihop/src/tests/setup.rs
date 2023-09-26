use crate::contract::{Multihop, MultihopClient};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Env};
use crate::factory_contract;

pub fn deploy_factory<'a>(env: &Env, admin: &Address) -> factory_contract::Client<'a> {
    factory_contract::Client::new(env, &env.register_stellar_asset_contract(admin.clone()))
}

pub fn deploy_multihop_contract<'a>(
    env: &Env,
    admin: impl Into<Option<Address>>,
    factory: Address,
) -> MultihopClient<'a> {
    let admin = admin.into().unwrap_or(Address::random(env));

    let multihop = MultihopClient::new(env, &env.register_contract(None, Multihop {}));

    multihop.initialize(&admin, &factory);
    multihop
}
