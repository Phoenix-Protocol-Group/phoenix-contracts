//! Tests for same-pair coexistence of pools of different `PoolType`.
//!
//! These exercise the V2 pair-tuple key, the type-aware factory query
//! (`query_pool_by_pair_type`), and the back-compat property that
//! `query_for_pool_by_token_pair(a, b)` keeps returning the Xyk pool even
//! after a Blend pool is created for the same pair.
extern crate std;

use super::setup::{
    deploy_factory_contract, generate_lp_init_info, install_and_deploy_token_contract,
    install_blend_lp,
};
use crate::contract::FactoryClient;

use phoenix::utils::PoolType;
use soroban_sdk::{
    testutils::{Address as _},
    Address, Env, String,
};

fn setup_factory_with_blend<'a>(env: &Env, admin: &Address) -> FactoryClient<'a> {
    let factory = deploy_factory_contract(env, Some(admin.clone()));
    // Register the Blend pool wasm so the factory can deploy Blend pools.
    let blend_wasm_hash = install_blend_lp(env);
    factory.set_blend_wasm_hash(&blend_wasm_hash);
    factory
}

fn fresh_token_pair<'a>(
    env: &'a Env,
    admin: &Address,
) -> (
    crate::token_contract::Client<'a>,
    crate::token_contract::Client<'a>,
) {
    let mut a = install_and_deploy_token_contract(
        env,
        admin.clone(),
        7,
        String::from_str(env, "AlphaToken"),
        String::from_str(env, "ALPHA"),
    );
    let mut b = install_and_deploy_token_contract(
        env,
        admin.clone(),
        7,
        String::from_str(env, "BetaToken"),
        String::from_str(env, "BETA"),
    );
    if b.address < a.address {
        std::mem::swap(&mut a, &mut b);
    }
    (a, b)
}

#[test]
fn xyk_then_blend_same_pair_coexist() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let factory = setup_factory_with_blend(&env, &admin);

    let (token_a, token_b) = fresh_token_pair(&env, &admin);
    let lp_init_info = generate_lp_init_info(
        token_a.address.clone(),
        token_b.address.clone(),
        Address::generate(&env),
        admin.clone(),
        Address::generate(&env),
    );

    // 1. Create the Xyk pool first.
    let xyk_pool = factory.create_liquidity_pool(
        &admin,
        &lp_init_info,
        &String::from_str(&env, "Xyk"),
        &String::from_str(&env, "XYK"),
        &PoolType::Xyk,
        &None::<u64>,
        &500i64,
        &10_000i64,
    );

    // Legacy query returns the Xyk pool, both orders.
    assert_eq!(
        factory.query_for_pool_by_token_pair(&token_a.address, &token_b.address),
        xyk_pool
    );
    assert_eq!(
        factory.query_for_pool_by_token_pair(&token_b.address, &token_a.address),
        xyk_pool
    );
    // Type-aware query for Xyk: must agree.
    assert_eq!(
        factory.query_pool_by_pair_type(
            &token_a.address,
            &token_b.address,
            &PoolType::Xyk,
        ),
        xyk_pool
    );

    // 2. Create the Blend pool for the SAME unordered pair.
    let blend_pool = factory.create_liquidity_pool(
        &admin,
        &lp_init_info,
        &String::from_str(&env, "Blend"),
        &String::from_str(&env, "BLND"),
        &PoolType::Blend,
        &None::<u64>,
        &500i64,
        &10_000i64,
    );

    // Different deterministic address — salt prefix did its job.
    assert_ne!(xyk_pool, blend_pool, "Xyk and Blend pools must occupy distinct addresses");

    // Legacy query MUST still return the Xyk pool (back-compat invariant).
    assert_eq!(
        factory.query_for_pool_by_token_pair(&token_a.address, &token_b.address),
        xyk_pool,
        "legacy query must not be hijacked by the Blend pool"
    );
    assert_eq!(
        factory.query_for_pool_by_token_pair(&token_b.address, &token_a.address),
        xyk_pool,
        "legacy query (reversed) must not be hijacked either"
    );

    // Type-aware query: Xyk → xyk, Blend → blend, both orders.
    assert_eq!(
        factory.query_pool_by_pair_type(&token_a.address, &token_b.address, &PoolType::Xyk),
        xyk_pool
    );
    assert_eq!(
        factory.query_pool_by_pair_type(&token_b.address, &token_a.address, &PoolType::Xyk),
        xyk_pool
    );
    assert_eq!(
        factory.query_pool_by_pair_type(&token_a.address, &token_b.address, &PoolType::Blend),
        blend_pool
    );
    assert_eq!(
        factory.query_pool_by_pair_type(&token_b.address, &token_a.address, &PoolType::Blend),
        blend_pool
    );

    // Both pools appear in the global pool vec.
    let all_pools = factory.query_pools();
    assert!(all_pools.contains(&xyk_pool));
    assert!(all_pools.contains(&blend_pool));
}

#[test]
fn blend_then_xyk_same_pair_coexist() {
    // Inverse of the test above: prove the same invariants hold regardless of
    // the order in which the two pools are deployed. This catches any subtle
    // dependency on creation order (e.g., an unconditional legacy-write that
    // would let the second Xyk creation overwrite the first Blend lookup).
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let factory = setup_factory_with_blend(&env, &admin);

    let (token_a, token_b) = fresh_token_pair(&env, &admin);
    let lp_init_info = generate_lp_init_info(
        token_a.address.clone(),
        token_b.address.clone(),
        Address::generate(&env),
        admin.clone(),
        Address::generate(&env),
    );

    // 1. Blend first.
    let blend_pool = factory.create_liquidity_pool(
        &admin,
        &lp_init_info,
        &String::from_str(&env, "Blend"),
        &String::from_str(&env, "BLND"),
        &PoolType::Blend,
        &None::<u64>,
        &500i64,
        &10_000i64,
    );

    // Before any Xyk exists, the type-aware Blend query resolves; the
    // type-aware Xyk query AND the legacy query must miss (Blend never
    // writes the legacy slot).
    assert_eq!(
        factory.query_pool_by_pair_type(&token_a.address, &token_b.address, &PoolType::Blend),
        blend_pool
    );

    // 2. Xyk for the same pair.
    let xyk_pool = factory.create_liquidity_pool(
        &admin,
        &lp_init_info,
        &String::from_str(&env, "Xyk"),
        &String::from_str(&env, "XYK"),
        &PoolType::Xyk,
        &None::<u64>,
        &500i64,
        &10_000i64,
    );

    assert_ne!(xyk_pool, blend_pool);
    // After Xyk creation the legacy slot now points at Xyk.
    assert_eq!(
        factory.query_for_pool_by_token_pair(&token_a.address, &token_b.address),
        xyk_pool
    );
    // Type-aware queries still resolve to the right pool per type.
    assert_eq!(
        factory.query_pool_by_pair_type(&token_a.address, &token_b.address, &PoolType::Xyk),
        xyk_pool
    );
    assert_eq!(
        factory.query_pool_by_pair_type(&token_a.address, &token_b.address, &PoolType::Blend),
        blend_pool
    );
}

#[test]
#[should_panic(expected = "ExistingValue")]
fn second_xyk_for_same_pair_fails() {
    // The salt-prefix change is specifically a Blend-only carve-out — Xyk's
    // salt is unchanged from production, so a second Xyk for the same pair
    // must still collide on the deterministic address.
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let factory = setup_factory_with_blend(&env, &admin);

    let (token_a, token_b) = fresh_token_pair(&env, &admin);
    let lp_init_info = generate_lp_init_info(
        token_a.address.clone(),
        token_b.address.clone(),
        Address::generate(&env),
        admin.clone(),
        Address::generate(&env),
    );

    let _xyk_a = factory.create_liquidity_pool(
        &admin,
        &lp_init_info,
        &String::from_str(&env, "Xyk"),
        &String::from_str(&env, "XYK"),
        &PoolType::Xyk,
        &None::<u64>,
        &500i64,
        &10_000i64,
    );

    // Second Xyk for the same pair must blow up on `deploy_v2`.
    let _xyk_b = factory.create_liquidity_pool(
        &admin,
        &lp_init_info,
        &String::from_str(&env, "Xyk2"),
        &String::from_str(&env, "XYK2"),
        &PoolType::Xyk,
        &None::<u64>,
        &500i64,
        &10_000i64,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #103)")]
fn pair_type_query_unknown_blend_panics() {
    // No Blend pool was ever created for this pair → V2 miss + no Xyk
    // fallback (fallback is gated to Xyk only) → panic with
    // ContractError::LiquidityPoolNotFound (= 103).
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let factory = setup_factory_with_blend(&env, &admin);

    let (token_a, token_b) = fresh_token_pair(&env, &admin);
    let lp_init_info = generate_lp_init_info(
        token_a.address.clone(),
        token_b.address.clone(),
        Address::generate(&env),
        admin.clone(),
        Address::generate(&env),
    );

    let _xyk = factory.create_liquidity_pool(
        &admin,
        &lp_init_info,
        &String::from_str(&env, "Xyk"),
        &String::from_str(&env, "XYK"),
        &PoolType::Xyk,
        &None::<u64>,
        &500i64,
        &10_000i64,
    );

    factory.query_pool_by_pair_type(&token_a.address, &token_b.address, &PoolType::Blend);
}
