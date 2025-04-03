use soroban_sdk::{Address, BytesN, Env};

use crate::token_contract;

#[allow(clippy::too_many_arguments)]
pub mod old_vesting {
    soroban_sdk::contractimport!(file = "../../.artifacts_sdk_update/old_phoenix_vesting.wasm");
}

pub fn install_latest_vesting(env: &Env) -> BytesN<32> {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/phoenix_vesting.wasm"
    );
    env.deployer().upload_contract_wasm(WASM)
}

pub fn deploy_token_contract<'a>(env: &Env, admin: &Address) -> token_contract::Client<'a> {
    token_contract::Client::new(
        env,
        &env.register_stellar_asset_contract_v2(admin.clone())
            .address(),
    )
}
