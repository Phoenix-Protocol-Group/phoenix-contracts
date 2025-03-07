use crate::storage::{DataKey, CONFIG};
use certora_soroban::is_auth;
use cvlr::asserts::cvlr_satisfy;
use cvlr_soroban_derive::rule;
use soroban_sdk::Env;

use crate::contract::{LiquidityPool, LiquidityPoolTrait};

#[rule]
fn sanity() {
    cvlr_satisfy!(true);
}

#[rule]
fn certora_only_admin_can_update_config(env: Env, total_fee_bps: i64) {
    LiquidityPool::update_config(env, None, Some(total_fee_bps), None, None, None, None);
    certora::satisfy!(false);
}
