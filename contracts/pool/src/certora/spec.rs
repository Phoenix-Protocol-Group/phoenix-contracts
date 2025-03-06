use crate::storage::CONFIG;
use cvlr::asserts::cvlr_satisfy;
use cvlr_soroban_derive::rule;
use soroban_sdk::Env;

use crate::contract::{LiquidityPool, LiquidityPoolTrait};

#[rule]
fn sanity() {
    cvlr_satisfy!(true);
}

#[rule]
fn certora_query_config(env: Env) {
    certora::require!(env.storage().persistent().has(&CONFIG), "config exists");
    let _config = LiquidityPool::query_config(env);
    certora::satisfy!(true);
}
