use soroban_sdk::{Address, Env};

use crate::{
    contract::{Vesting, VestingClient},
    storage::{MinterInfo, VestingTokenInfo},
    token_contract,
};

pub fn instantiate_vesting_client<'a>(
    env: &Env,
    admin: &Address,
    vesting_token: VestingTokenInfo,
    max_vesting_complexity: u32,
    minter_info: Option<MinterInfo>,
) -> VestingClient<'a> {
    VestingClient::new(
        env,
        &env.register(
            Vesting,
            (
                admin.clone(),
                vesting_token,
                max_vesting_complexity,
                minter_info,
            ),
        ),
    )
}

pub fn deploy_token_contract<'a>(env: &Env, admin: &Address) -> token_contract::Client<'a> {
    token_contract::Client::new(
        env,
        &env.register_stellar_asset_contract_v2(admin.clone())
            .address(),
    )
}
