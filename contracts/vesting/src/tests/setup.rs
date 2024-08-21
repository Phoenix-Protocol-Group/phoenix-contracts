use soroban_sdk::{Address, Env};

use crate::{
    contract::{Vesting, VestingClient},
    token_contract,
};

pub fn instantiate_vesting_client(env: &Env) -> VestingClient {
    VestingClient::new(env, &env.register_contract(None, Vesting {}))
}

pub fn deploy_token_contract<'a>(env: &Env, admin: &Address) -> token_contract::Client<'a> {
    token_contract::Client::new(
        env,
        &env.register_stellar_asset_contract_v2(admin.clone())
            .address(),
    )
}
