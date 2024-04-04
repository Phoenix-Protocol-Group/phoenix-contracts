use soroban_sdk::Env;

use crate::contract::{Vesting, VestingClient};

pub fn instantiate_vesting_client(env: &Env) -> VestingClient {
    VestingClient::new(env, &env.register_contract(None, Vesting {}))
}
