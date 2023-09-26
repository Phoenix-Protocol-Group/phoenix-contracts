use crate::contract::{Multihop, MultihopClient};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Bytes, BytesN, Env};
use soroban_sdk::arbitrary::std::dbg;
use crate::factory_contract;

pub fn deploy_and_init_factory_contract<'a>(env: &Env, admin: &Address) -> factory_contract::Client<'a> {
    let factory_client = factory_contract::Client::new(env, &env.register_stellar_asset_contract(admin.clone()));
    factory_client.initialize(admin);
    factory_client
}

pub fn factory_client<'a>(env: &Env, admin: &Address) -> factory_contract::Client<'a> {
    factory_contract::Client::new(env, &env.register_stellar_asset_contract(admin.clone()))
}

pub fn deploy_factory_contract_from_wasm(e: &Env) -> Address {
    let deployer = e.current_contract_address();

    if deployer != e.current_contract_address() {
        deployer.require_auth();
    }

    let salt = Bytes::new(e);
    let salt = e.crypto().sha256(&salt);

    let factory_wasm = install_factory_wasm(&e);
    e.deployer()
        .with_address(deployer, salt)
        .deploy(factory_wasm)
}

pub fn install_factory_wasm(env: &Env) -> BytesN<32> {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/phoenix_factory.wasm"
    );
    env.deployer().upload_contract_wasm(WASM)
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
