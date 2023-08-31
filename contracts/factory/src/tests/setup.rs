use crate::contract::{Factory, FactoryClient};
use phoenix::token_contract;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, BytesN, Env};

#[allow(clippy::too_many_arguments)]
pub fn deploy_token_contract<'a>(env: &Env, admin: &Address) -> token_contract::Client<'a> {
    token_contract::Client::new(env, &env.register_stellar_asset_contract(admin.clone()))
}

#[allow(clippy::too_many_arguments)]
pub fn install_lp_contract(env: &Env) -> BytesN<32> {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/phoenix_pair.wasm"
    );
    env.deployer().upload_contract_wasm(WASM)
}

pub fn install_token_wasm(env: &Env) -> BytesN<32> {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/soroban_token_contract.wasm"
    );
    env.deployer().upload_contract_wasm(WASM)
}

pub fn install_stake_wasm(env: &Env) -> BytesN<32> {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/phoenix_stake.wasm"
    );
    env.deployer().upload_contract_wasm(WASM)
}

pub fn deploy_factory_contract<'a>(
    env: &Env,
    admin: impl Into<Option<Address>>,
) -> FactoryClient<'a> {
    let admin = admin.into().unwrap_or(Address::random(env));
    let factory = FactoryClient::new(env, &env.register_contract(None, Factory {}));

    factory.initialize(&admin);
    factory
}
