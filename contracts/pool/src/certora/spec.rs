use soroban_sdk::Env;

use crate::contract::LiquidityPool;
use crate::contract::LiquidityPoolTrait;

use cvlr::asserts::cvlr_satisfy;
use cvlr_soroban_derive::rule;

#[rule]
fn sanity(e: Env) {
    //let _ = LiquidityPool::query_pool_info(e.clone());
    cvlr_satisfy!(true);
}
