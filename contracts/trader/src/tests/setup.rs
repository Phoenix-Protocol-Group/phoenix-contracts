use soroban_sdk::{Address, Env};

use crate::{
    contract::{Trader, TraderClient},
    lp_contract, token_contract,
};

pub fn deploy_token_client(env: &Env, token_address: Address) -> token_contract::Client {
    token_contract::Client::new(env, &token_address)
}

pub fn deploy_lp_client(env: &Env, lp_address: Address) -> lp_contract::Client {
    lp_contract::Client::new(env, &lp_address)
}

pub fn deploy_trader_client(env: &Env) -> TraderClient {
    TraderClient::new(env, &env.register_contract(None, Trader {}))
}
