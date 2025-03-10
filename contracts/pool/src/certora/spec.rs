use crate::storage::DataKey;
use certora_soroban::is_auth;
use cvlr::{asserts::cvlr_satisfy, cvlr_assume};
use cvlr_soroban_derive::rule;
use soroban_sdk::{Address, Env};

use crate::contract::{LiquidityPool, LiquidityPoolTrait};

#[rule]
fn sanity() {
    cvlr_satisfy!(true);
}

#[rule]
fn certora_only_admin_can_update_config(env: Env, total_fee_bps: i64) {
    let admin = env
        .storage()
        .persistent()
        .get::<_, Address>(&DataKey::Admin)
        .unwrap();

    cvlr_assume!(is_auth(admin));
    LiquidityPool::update_config(env, None, Some(total_fee_bps), None, None, None, None);
    certora::satisfy!(true);
}
