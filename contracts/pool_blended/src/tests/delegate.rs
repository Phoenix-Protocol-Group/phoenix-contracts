extern crate std;

use pretty_assertions::assert_eq;
use soroban_sdk::{testutils::Address as _, Address, Env};

use super::setup::{deploy_liquidity_pool_contract, deploy_token_contract};
use crate::token_contract;

const ONE_M: i128 = 1_000_000_0000000; // 1,000,000 with 7 decimals
const HALF_M: i128 = 500_000_0000000;

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

    let swap_in: i128 = 1_000_0000000;
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

    // Pool now has 2M each side, two LPs with equal shares (minus the burned
    // 1000 minimum-liquidity shares on lp1 from pool bootstrap).

    // Mint donation amount to the delegate and donate it to side B.
    let donation: i128 = 100_000_0000000; // 100k
    h.token_b.mint(&h.delegate, &donation);

    let total_shares_before = h.pool.query_total_issued_lp();
    h.pool.donate(&h.token_b.address, &donation);
    let total_shares_after = h.pool.query_total_issued_lp();
    assert_eq!(
        total_shares_before, total_shares_after,
        "donate must not mint LP"
    );

    // Both LPs withdraw their entire stake.
    let lp1_shares = token_contract::Client::new(&h.env, &h.pool.query_share_token_address())
        .balance(&h.lp);
    let lp2_shares = token_contract::Client::new(&h.env, &h.pool.query_share_token_address())
        .balance(&lp2);
    assert_eq!(lp1_shares, lp2_shares, "equal stakes => equal shares");

    let lp1_b_before = h.token_b.balance(&h.lp);
    let lp2_b_before = h.token_b.balance(&lp2);

    h.pool
        .withdraw_liquidity(&h.lp, &lp1_shares, &0, &0, &None::<u64>, &None);
    h.pool
        .withdraw_liquidity(&lp2, &lp2_shares, &0, &0, &None::<u64>, &None);

    let lp1_gain_b = h.token_b.balance(&h.lp) - lp1_b_before;
    let lp2_gain_b = h.token_b.balance(&lp2) - lp2_b_before;

    // Each LP should get back their original ONE_M + ~half of the donation
    // (minus tiny rounding on the burned MINIMUM_LIQUIDITY shares).
    let expected_per_lp = ONE_M + donation / 2;
    let tolerance = 100_i128; // accept dust from integer division
    assert!(
        (lp1_gain_b - expected_per_lp).abs() <= tolerance,
        "lp1 gain {} should be ~{}",
        lp1_gain_b,
        expected_per_lp,
    );
    assert!(
        (lp2_gain_b - expected_per_lp).abs() <= tolerance,
        "lp2 gain {} should be ~{}",
        lp2_gain_b,
        expected_per_lp,
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
