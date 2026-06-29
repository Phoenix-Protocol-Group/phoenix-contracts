extern crate std;

use pretty_assertions::assert_eq;
use soroban_sdk::{testutils::Address as _, Address, Env};

use super::setup::{deploy_liquidity_pool_contract, deploy_token_contract};
use crate::token_contract;

const ONE_M: i128 = 10_000_000_000_000; // 1,000,000 with 7 decimals

/// Helper: spin up a pool with two tokens (sorted so A < B), an admin, and a
/// delegate; provide initial liquidity from `lp`. Returns the configured
/// addresses so the test can reach back in.
struct Harness<'a> {
    env: Env,
    pool: crate::contract::LiquidityPoolClient<'a>,
    token_a: token_contract::Client<'a>,
    token_b: token_contract::Client<'a>,
    admin: Address,
    delegate: Address,
    lp: Address,
}

fn setup() -> Harness<'static> {
    let env: Env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let mut admin_a = Address::generate(&env);
    let mut admin_b = Address::generate(&env);

    let mut token_a = deploy_token_contract(&env, &admin_a);
    let mut token_b = deploy_token_contract(&env, &admin_b);
    if token_b.address < token_a.address {
        std::mem::swap(&mut token_a, &mut token_b);
        std::mem::swap(&mut admin_a, &mut admin_b);
    }

    let pool_admin = Address::generate(&env);
    let delegate = Address::generate(&env);
    let lp = Address::generate(&env);
    let stake_manager = Address::generate(&env);
    let stake_owner = Address::generate(&env);

    let pool = deploy_liquidity_pool_contract(
        &env,
        Some(pool_admin.clone()),
        (&token_a.address, &token_b.address),
        0i64,
        None,
        None,
        None,
        stake_manager,
        stake_owner,
    );

    token_a.mint(&lp, &(ONE_M * 10));
    token_b.mint(&lp, &(ONE_M * 10));

    pool.provide_liquidity(
        &lp,
        &Some(ONE_M),
        &Some(ONE_M),
        &Some(ONE_M),
        &Some(ONE_M),
        &None,
        &None::<u64>,
        &false,
    );

    pool.set_delegate(&Some(delegate.clone()));

    Harness {
        env,
        pool,
        token_a,
        token_b,
        admin: pool_admin,
        delegate,
        lp,
    }
}

#[test]
fn set_and_query_delegate() {
    let h = setup();
    let state = h.pool.query_delegate_state();
    assert_eq!(state.delegate, Some(h.delegate.clone()));
    assert_eq!(state.delegated_a, 0);
    assert_eq!(state.delegated_b, 0);
    assert_eq!(state.total_a, ONE_M);
    assert_eq!(state.total_b, ONE_M);
    assert_eq!(state.liquid_a, ONE_M);
    assert_eq!(state.liquid_b, ONE_M);
}

#[test]
fn clear_delegate_unsets_it() {
    let h = setup();
    h.pool.set_delegate(&None);
    let state = h.pool.query_delegate_state();
    assert_eq!(state.delegate, None);
}

#[test]
fn pull_does_not_change_simulated_swap_price() {
    let h = setup();

    let swap_in: i128 = 10_000_000_000; // 1,000 with 7 decimals
    let pre = h.pool.simulate_swap(&h.token_a.address, &swap_in);

    // Pull a quarter of the B-side reserve out to the delegate.
    h.pool
        .withdraw_to_delegate(&h.token_b.address, &(ONE_M / 4));

    let post = h.pool.simulate_swap(&h.token_a.address, &swap_in);
    assert_eq!(pre.ask_amount, post.ask_amount);
    assert_eq!(pre.spread_amount, post.spread_amount);
    assert_eq!(pre.commission_amount, post.commission_amount);

    let state = h.pool.query_delegate_state();
    assert_eq!(state.delegated_b, ONE_M / 4);
    assert_eq!(state.liquid_b, ONE_M - ONE_M / 4);
    assert_eq!(state.total_b, ONE_M);
}

#[test]
fn pull_then_push_is_net_zero() {
    let h = setup();
    let amount = ONE_M / 5;

    h.pool.withdraw_to_delegate(&h.token_a.address, &amount);
    h.pool.deposit_from_delegate(&h.token_a.address, &amount);

    let state = h.pool.query_delegate_state();
    assert_eq!(state.delegated_a, 0);
    assert_eq!(state.liquid_a, ONE_M);
    assert_eq!(state.total_a, ONE_M);
}

#[test]
fn donate_increases_lp_redemption_pro_rata() {
    let h = setup();

    // Second LP joins with equal stake.
    let lp2 = Address::generate(&h.env);
    h.token_a.mint(&lp2, &ONE_M);
    h.token_b.mint(&lp2, &ONE_M);
    h.pool.provide_liquidity(
        &lp2,
        &Some(ONE_M),
        &Some(ONE_M),
        &Some(ONE_M),
        &Some(ONE_M),
        &None,
        &None::<u64>,
        &false,
    );

    // Pool now has 2M each side. LP1 bootstrapped the pool so
    // `MINIMUM_LIQUIDITY_AMOUNT = 1000` shares were minted to the pool
    // itself (Uniswap-style first-deposit burn). LP2's deposit minted
    // exactly `ONE_M` shares on top. Total supply = `2 * ONE_M`; LP1 owns
    // `ONE_M - 1000`, LP2 owns `ONE_M`, pool owns `1000` permanently.

    // Mint donation amount to the delegate and donate it to side B.
    let donation: i128 = 1_000_000_000_000; // 100k with 7 decimals
    h.token_b.mint(&h.delegate, &donation);

    let total_shares_before = h.pool.query_total_issued_lp();
    h.pool.donate(&h.token_b.address, &donation);
    let total_shares_after = h.pool.query_total_issued_lp();
    assert_eq!(
        total_shares_before, total_shares_after,
        "donate must not mint LP"
    );

    let share_token = h.pool.query_share_token_address();
    let lp1_shares = token_contract::Client::new(&h.env, &share_token).balance(&h.lp);
    let lp2_shares = token_contract::Client::new(&h.env, &share_token).balance(&lp2);
    // LP1 is the bootstrap LP so it is short MINIMUM_LIQUIDITY_AMOUNT shares.
    // The two LPs deposited equal stakes; the only difference is the burn.
    const MINIMUM_LIQUIDITY_AMOUNT: i128 = 1_000;
    assert_eq!(
        lp2_shares - lp1_shares,
        MINIMUM_LIQUIDITY_AMOUNT,
        "bootstrap LP should be short MINIMUM_LIQUIDITY_AMOUNT shares",
    );

    let lp1_b_before = h.token_b.balance(&h.lp);
    let lp2_b_before = h.token_b.balance(&lp2);

    h.pool
        .withdraw_liquidity(&h.lp, &lp1_shares, &0, &0, &None::<u64>, &None);
    h.pool
        .withdraw_liquidity(&lp2, &lp2_shares, &0, &0, &None::<u64>, &None);

    let lp1_gain_b = h.token_b.balance(&h.lp) - lp1_b_before;
    let lp2_gain_b = h.token_b.balance(&lp2) - lp2_b_before;

    // Each LP should get back ~ONE_M + ~half of the donation. Tolerance
    // accounts for: the 1000 shares permanently locked in the pool (dilutes
    // each LP by ~MIN/total_supply of the post-donate pool) plus integer
    // rounding on the share-ratio math. Empirically the deviation is on
    // the order of MIN * (1 + donation/(2*ONE_M)) ~= 1050; we accept 5000.
    let expected_per_lp = ONE_M + donation / 2;
    let tolerance: i128 = 5_000;
    assert!(
        (lp1_gain_b - expected_per_lp).abs() <= tolerance,
        "lp1 gain {} should be within {} of ~{}",
        lp1_gain_b,
        tolerance,
        expected_per_lp,
    );
    assert!(
        (lp2_gain_b - expected_per_lp).abs() <= tolerance,
        "lp2 gain {} should be within {} of ~{}",
        lp2_gain_b,
        tolerance,
        expected_per_lp,
    );
    // LP2 should get back slightly more than LP1 since LP1 carried the
    // bootstrap dust loss.
    assert!(
        lp2_gain_b > lp1_gain_b,
        "lp2 ({}) should out-gain lp1 ({}) by the bootstrap dust",
        lp2_gain_b,
        lp1_gain_b,
    );
}

#[test]
fn provide_liquidity_with_delegated_out_credits_correctly() {
    let h = setup();

    // Pull a chunk from side B so logical > physical for side B.
    let parked = ONE_M / 4;
    h.pool.withdraw_to_delegate(&h.token_b.address, &parked);

    // Logical reserves remain (ONE_M, ONE_M); a new LP depositing at that ratio
    // should receive shares proportional to (deposit / logical_reserve), not
    // (deposit / physical_balance).
    let lp2 = Address::generate(&h.env);
    let deposit = ONE_M / 10;
    h.token_a.mint(&lp2, &deposit);
    h.token_b.mint(&lp2, &deposit);

    let total_shares_before = h.pool.query_total_issued_lp();
    h.pool.provide_liquidity(
        &lp2,
        &Some(deposit),
        &Some(deposit),
        &Some(deposit),
        &Some(deposit),
        &None,
        &None::<u64>,
        &false,
    );
    let total_shares_after = h.pool.query_total_issued_lp();
    let minted = total_shares_after - total_shares_before;

    // Expected: minted ≈ (deposit * total_shares_before) / ONE_M
    let expected = deposit * total_shares_before / ONE_M;
    let tolerance = 10_i128;
    assert!(
        (minted - expected).abs() <= tolerance,
        "minted {} should be ≈ {} (logical-reserve denominator)",
        minted,
        expected,
    );

    // Logical reserves grew by `deposit` per side; delegated_out unchanged.
    let state = h.pool.query_delegate_state();
    assert_eq!(state.total_a, ONE_M + deposit);
    assert_eq!(state.total_b, ONE_M + deposit);
    assert_eq!(state.delegated_b, parked);
}

#[test]
#[should_panic(expected = "Error(Contract, #333)")] // DelegateNotSet
fn withdraw_when_delegate_unset_rejects() {
    let h = setup();
    h.pool.set_delegate(&None);
    h.pool.withdraw_to_delegate(&h.token_a.address, &1_000_000);
}

#[test]
#[should_panic(expected = "Error(Contract, #333)")]
fn deposit_when_delegate_unset_rejects() {
    let h = setup();
    h.pool.set_delegate(&None);
    h.pool.deposit_from_delegate(&h.token_a.address, &1_000_000);
}

#[test]
#[should_panic(expected = "Error(Contract, #333)")]
fn donate_when_delegate_unset_rejects() {
    let h = setup();
    h.pool.set_delegate(&None);
    h.pool.donate(&h.token_a.address, &1_000_000);
}

#[test]
#[should_panic(expected = "Error(Contract, #334)")] // DelegateUnauthorizedToken
fn withdraw_with_wrong_token_rejects() {
    let h = setup();
    let stray_admin = Address::generate(&h.env);
    let stray = deploy_token_contract(&h.env, &stray_admin);
    h.pool.withdraw_to_delegate(&stray.address, &1_000_000);
}

#[test]
#[should_panic(expected = "Error(Contract, #335)")] // DelegatedOutUnderflow
fn deposit_more_than_delegated_rejects() {
    let h = setup();
    let parked = ONE_M / 10;
    h.pool.withdraw_to_delegate(&h.token_a.address, &parked);
    // Try to return double; should underflow the DelegatedOutA counter.
    h.pool
        .deposit_from_delegate(&h.token_a.address, &(parked * 2));
}

#[test]
#[should_panic(expected = "Error(Contract, #336)")] // DelegateInvalidAmount
fn withdraw_zero_rejects() {
    let h = setup();
    h.pool.withdraw_to_delegate(&h.token_a.address, &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #336)")]
fn donate_negative_rejects() {
    let h = setup();
    h.pool.donate(&h.token_b.address, &-1);
}

/// Sanity: query_delegate_state reflects mid-flight delegate balances.
#[test]
fn query_delegate_state_tracks_delegated_amounts() {
    let h = setup();
    let parked_a = ONE_M / 3;
    let parked_b = ONE_M / 7;

    h.pool.withdraw_to_delegate(&h.token_a.address, &parked_a);
    h.pool.withdraw_to_delegate(&h.token_b.address, &parked_b);

    let state = h.pool.query_delegate_state();
    assert_eq!(state.delegated_a, parked_a);
    assert_eq!(state.delegated_b, parked_b);
    assert_eq!(state.liquid_a, ONE_M - parked_a);
    assert_eq!(state.liquid_b, ONE_M - parked_b);
    assert_eq!(state.total_a, ONE_M);
    assert_eq!(state.total_b, ONE_M);

    // Quiet "unused field" warnings on the harness without touching the rest
    // of the test surface.
    let _ = h.admin;
}

/// query_pool_info must return the LOGICAL reserve (= physical balance +
/// delegated_out), unchanged by delegate movements. Factory/multihop
/// callers depend on this invariant for pricing math.
#[test]
fn query_pool_info_preserves_logical_reserve_under_delegation() {
    let h = setup();

    let before = h.pool.query_pool_info();
    assert_eq!(before.asset_a.amount, ONE_M);
    assert_eq!(before.asset_b.amount, ONE_M);

    let parked_a = ONE_M / 4;
    let parked_b = ONE_M / 3;
    h.pool.withdraw_to_delegate(&h.token_a.address, &parked_a);
    h.pool.withdraw_to_delegate(&h.token_b.address, &parked_b);

    let mid = h.pool.query_pool_info();
    assert_eq!(
        mid.asset_a.amount, before.asset_a.amount,
        "delegate withdraw must not shrink logical reserve A",
    );
    assert_eq!(
        mid.asset_b.amount, before.asset_b.amount,
        "delegate withdraw must not shrink logical reserve B",
    );

    // Cross-check: the logical reserve from query_pool_info equals the
    // total_{a,b} reported by query_delegate_state.
    let state = h.pool.query_delegate_state();
    assert_eq!(mid.asset_a.amount, state.total_a);
    assert_eq!(mid.asset_b.amount, state.total_b);
    // And `liquid + delegated == total` on each side.
    assert_eq!(state.liquid_a + state.delegated_a, state.total_a);
    assert_eq!(state.liquid_b + state.delegated_b, state.total_b);

    // Return-then-donate cycle on side B: logical reserve grows by the
    // donation, regardless of the delegated leg returning first.
    h.pool.deposit_from_delegate(&h.token_b.address, &parked_b);
    let donation: i128 = 500_000_000_000; // 50k with 7 decimals
    h.token_b.mint(&h.delegate, &donation);
    h.pool.donate(&h.token_b.address, &donation);

    let after = h.pool.query_pool_info();
    assert_eq!(
        after.asset_a.amount, before.asset_a.amount,
        "side A logical reserve unchanged across roundtrip on B",
    );
    assert_eq!(
        after.asset_b.amount,
        before.asset_b.amount + donation,
        "donate must increase logical reserve by exactly the donated amount",
    );
}
