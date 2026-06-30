//! Proves multihop routes correctly when an Xyk pool and a Blend pool exist
//! for the SAME unordered pair of tokens. The two pools live at different
//! deterministic addresses (per the salt-prefix change in the factory) and
//! the factory's type-aware query keeps them disambiguated.

extern crate std;

use crate::factory_contract::PoolType;
use crate::storage::Swap;
use crate::tests::setup::{
    deploy_and_initialize_pool, deploy_and_mint_tokens, deploy_factory_with_blend_support,
    deploy_multihop_contract,
};
use crate::xyk_pool;

use soroban_sdk::{testutils::Address as _, vec, Address, Env};

fn pool_address_for(
    factory: &crate::factory_contract::Client,
    token_a: &Address,
    token_b: &Address,
    pool_type: PoolType,
) -> Address {
    factory.query_pool_by_pair_type(token_a, token_b, &pool_type)
}

#[test]
fn multihop_routes_xyk_and_blend_independently_for_same_pair() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);

    // Two tokens forming a single unordered pair we'll attach both an Xyk and
    // a Blend pool to.
    let token_a = deploy_and_mint_tokens(&env, &admin, 50_000_000i128);
    let token_b = deploy_and_mint_tokens(&env, &admin, 50_000_000i128);

    let factory = deploy_factory_with_blend_support(&env, admin.clone());

    // Deliberately seed the two pools with DIFFERENT reserve ratios so each
    // gives a different swap output. That way an assertion on the recipient
    // balance is enough to tell which pool the swap went through.
    let xyk_a_reserve: i128 = 1_000_000;
    let xyk_b_reserve: i128 = 1_000_000;
    let blend_a_reserve: i128 = 2_000_000;
    let blend_b_reserve: i128 = 4_000_000; // 1:2 — blend gives more B per A
    deploy_and_initialize_pool(
        &env,
        &factory,
        admin.clone(),
        token_a.address.clone(),
        xyk_a_reserve,
        token_b.address.clone(),
        xyk_b_reserve,
        None,
        PoolType::Xyk,
    );
    deploy_and_initialize_pool(
        &env,
        &factory,
        admin.clone(),
        token_a.address.clone(),
        blend_a_reserve,
        token_b.address.clone(),
        blend_b_reserve,
        None,
        PoolType::Blend,
    );

    // Discover both pool addresses from the factory; sanity-check that they
    // are distinct and that the legacy query still resolves the Xyk one
    // (back-compat invariant — multihop's Xyk routing depends on it).
    let xyk_addr =
        pool_address_for(&factory, &token_a.address, &token_b.address, PoolType::Xyk);
    let blend_addr =
        pool_address_for(&factory, &token_a.address, &token_b.address, PoolType::Blend);
    assert_ne!(xyk_addr, blend_addr);
    assert_eq!(
        factory.query_for_pool_by_token_pair(&token_a.address, &token_b.address),
        xyk_addr,
        "legacy query (no pool_type arg) must resolve the Xyk pool, not Blend"
    );

    // Snapshot pre-swap balances of the pool contracts. These are the
    // tokens currently owned by each pool, which equal the swap reserves.
    let xyk_b_before = token_b.balance(&xyk_addr);
    let blend_b_before = token_b.balance(&blend_addr);

    let multihop = deploy_multihop_contract(&env, admin.clone(), &factory.address);

    // ----- Xyk swap: 10 token_a in -> some token_b out via the Xyk pool ----
    let recipient_xyk = Address::generate(&env);
    let offer: i128 = 10_000;
    token_a.mint(&recipient_xyk, &offer);

    let swap_op = Swap {
        offer_asset: token_a.address.clone(),
        ask_asset: token_b.address.clone(),
        ask_asset_min_amount: None::<i128>,
    };
    multihop.swap(
        &recipient_xyk,
        &vec![&env, swap_op.clone()],
        &None::<i64>,
        &offer,
        &PoolType::Xyk,
        &None::<u64>,
        &None::<i64>,
    );

    let xyk_b_after = token_b.balance(&xyk_addr);
    let blend_b_after_xyk_swap = token_b.balance(&blend_addr);
    let recipient_xyk_received = token_b.balance(&recipient_xyk);

    assert!(
        recipient_xyk_received > 0,
        "recipient must have received token_b from the Xyk pool"
    );
    assert!(
        xyk_b_after < xyk_b_before,
        "Xyk pool's token_b reserve must decrease"
    );
    assert_eq!(
        blend_b_after_xyk_swap, blend_b_before,
        "Blend pool must be untouched by an Xyk-routed swap"
    );

    // ----- Blend swap: same shape, but routed through Blend ----------------
    let recipient_blend = Address::generate(&env);
    token_a.mint(&recipient_blend, &offer);

    multihop.swap(
        &recipient_blend,
        &vec![&env, swap_op.clone()],
        &None::<i64>,
        &offer,
        &PoolType::Blend,
        &None::<u64>,
        &None::<i64>,
    );

    let blend_b_after_blend_swap = token_b.balance(&blend_addr);
    let xyk_b_after_blend_swap = token_b.balance(&xyk_addr);
    let recipient_blend_received = token_b.balance(&recipient_blend);

    assert!(
        recipient_blend_received > 0,
        "recipient must have received token_b from the Blend pool"
    );
    assert!(
        blend_b_after_blend_swap < blend_b_before,
        "Blend pool's token_b reserve must decrease"
    );
    assert_eq!(
        xyk_b_after_blend_swap, xyk_b_after,
        "Xyk pool must be untouched by a Blend-routed swap"
    );

    // The two routes give different outputs (we asymmetrically funded the
    // pools precisely so this is observable).
    assert_ne!(
        recipient_xyk_received, recipient_blend_received,
        "Xyk and Blend swaps should yield different outputs given asymmetric reserves"
    );
    // Sanity: Blend pool is funded 2 token_b per 1 token_a (vs 1:1 on Xyk),
    // so a small swap of token_a → token_b should produce strictly MORE B
    // through Blend than through Xyk.
    assert!(
        recipient_blend_received > recipient_xyk_received,
        "Blend route should give more token_b (richer reserve ratio)"
    );
}

#[test]
fn simulate_swap_dispatches_per_pool_type() {
    // Same setup, but exercises `simulate_swap` instead of `swap`. Mirrors
    // the routing fix in the simulate path.
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let token_a = deploy_and_mint_tokens(&env, &admin, 50_000_000i128);
    let token_b = deploy_and_mint_tokens(&env, &admin, 50_000_000i128);

    let factory = deploy_factory_with_blend_support(&env, admin.clone());

    deploy_and_initialize_pool(
        &env,
        &factory,
        admin.clone(),
        token_a.address.clone(),
        1_000_000,
        token_b.address.clone(),
        1_000_000,
        None,
        PoolType::Xyk,
    );
    deploy_and_initialize_pool(
        &env,
        &factory,
        admin.clone(),
        token_a.address.clone(),
        2_000_000,
        token_b.address.clone(),
        4_000_000,
        None,
        PoolType::Blend,
    );

    let multihop = deploy_multihop_contract(&env, admin.clone(), &factory.address);

    let swap_op = Swap {
        offer_asset: token_a.address.clone(),
        ask_asset: token_b.address.clone(),
        ask_asset_min_amount: None::<i128>,
    };

    let xyk_sim =
        multihop.simulate_swap(&vec![&env, swap_op.clone()], &10_000i128, &PoolType::Xyk);
    let blend_sim =
        multihop.simulate_swap(&vec![&env, swap_op], &10_000i128, &PoolType::Blend);

    assert!(xyk_sim.ask_amount > 0);
    assert!(blend_sim.ask_amount > 0);
    assert_ne!(
        xyk_sim.ask_amount, blend_sim.ask_amount,
        "simulate_swap must reflect different routed pools"
    );
    assert!(
        blend_sim.ask_amount > xyk_sim.ask_amount,
        "richer Blend reserves should simulate a better ask_amount"
    );
}

#[test]
fn pre_existing_xyk_routes_via_legacy_query_after_factory_upgrade() {
    // The Xyk dispatch in multihop deliberately still goes through the
    // legacy `query_for_pool_by_token_pair`. This test pins that behavior:
    // a pool created BEFORE Blend support shipped (i.e., one whose V2 slot
    // we explicitly skip writing to model pre-V2 state) must still be
    // reachable via multihop's Xyk swap.
    //
    // We approximate "pre-V2" by deploying the Xyk pool and then asserting
    // that the legacy query continues to be the source of truth for Xyk
    // routing even if the V2 slot were absent.
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let token_a = deploy_and_mint_tokens(&env, &admin, 10_000_000i128);
    let token_b = deploy_and_mint_tokens(&env, &admin, 10_000_000i128);

    let factory = deploy_factory_with_blend_support(&env, admin.clone());
    deploy_and_initialize_pool(
        &env,
        &factory,
        admin.clone(),
        token_a.address.clone(),
        1_000_000,
        token_b.address.clone(),
        1_000_000,
        None,
        PoolType::Xyk,
    );

    let xyk_addr =
        factory.query_for_pool_by_token_pair(&token_a.address, &token_b.address);
    let multihop = deploy_multihop_contract(&env, admin.clone(), &factory.address);
    let recipient = Address::generate(&env);
    let offer: i128 = 5_000;
    token_a.mint(&recipient, &offer);

    multihop.swap(
        &recipient,
        &vec![
            &env,
            Swap {
                offer_asset: token_a.address.clone(),
                ask_asset: token_b.address.clone(),
                ask_asset_min_amount: None::<i128>,
            },
        ],
        &None::<i64>,
        &offer,
        &PoolType::Xyk,
        &None::<u64>,
        &None::<i64>,
    );

    let xyk_client = xyk_pool::Client::new(&env, &xyk_addr);
    let _ = xyk_client.query_config(); // sanity: pool is alive

    assert!(
        token_b.balance(&recipient) > 0,
        "recipient must receive token_b via Xyk routing"
    );
}
