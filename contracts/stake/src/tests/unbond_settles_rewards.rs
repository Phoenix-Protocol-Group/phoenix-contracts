//! Regression tests for the unbond-then-reward bug.
//!
//! Bug: `unbond` used to call `remove_stake` BEFORE settling pending
//! rewards. Because `calculate_pending_rewards` iterates the user's current
//! `stakes.stakes` vec, the rewards attributable to the just-removed stake
//! entry were unreachable forever — they stayed parked in the staking
//! contract's reward-token balance.
//!
//! The pool's `withdraw_liquidity(auto_unstake = …)` path calls
//! `stake.unbond` directly, so the same bug surfaced there too.
//!
//! Fix: settle the user's pending rewards (and roll `last_reward_time`
//! forward) inside `unbond` BEFORE mutating the stake vector. These tests
//! pin that behavior.
extern crate std;

use pretty_assertions::assert_eq;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, Vec,
};

use super::setup::{deploy_staking_contract, deploy_token_contract, ONE_DAY};
use crate::msg::StakedResponse;

const DEFAULT_COMPLEXITY: u32 = 7;
const SIXTY_DAYS: u64 = 60 * ONE_DAY;

#[test]
fn unbond_without_prior_withdraw_settles_pending_rewards() {
    // Single staker, single distribution period, then unbond. The user
    // never calls `withdraw_rewards` explicitly — `unbond` must pay them
    // anyway.
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let manager = Address::generate(&env);

    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);
    let staking = deploy_staking_contract(
        &env,
        admin.clone(),
        &lp_token.address,
        &manager,
        &admin,
        &DEFAULT_COMPLEXITY,
    );

    lp_token.mint(&user, &10_000);
    reward_token.mint(&admin, &20_000);

    let staked = 1_000;
    staking.bond(&user, &staked);

    // Move 60 days forward so the multiplier is at full 1.0.
    env.ledger().with_mut(|li| li.timestamp = SIXTY_DAYS);

    staking.create_distribution_flow(&admin, &reward_token.address);

    // Distribute 1000 reward tokens per day for 20 days.
    for _ in 0..20 {
        staking.distribute_rewards(&admin, &1_000, &reward_token.address);
        env.ledger().with_mut(|li| li.timestamp += ONE_DAY);
    }

    // Sanity: pending = 20_000, user has none yet.
    assert_eq!(
        staking
            .query_withdrawable_rewards(&user)
            .rewards
            .iter()
            .map(|r| r.reward_amount)
            .sum::<u128>(),
        20_000
    );
    assert_eq!(reward_token.balance(&user), 0);

    // Snapshot of reward-token balance held by the staking contract before
    // the unbond — used below to assert the rewards moved OUT of the
    // staking contract and into the user.
    let staking_reward_before = reward_token.balance(&staking.address);

    // The crux: unbond directly, WITHOUT a prior `withdraw_rewards`.
    staking.unbond(&user, &staked, &0);

    // Rewards must have landed with the user.
    assert_eq!(
        reward_token.balance(&user),
        20_000,
        "unbond must settle pending rewards instead of stranding them"
    );
    assert_eq!(
        reward_token.balance(&staking.address),
        staking_reward_before - 20_000,
        "the rewards must have left the staking contract"
    );

    // LP token returned, stake cleared.
    assert_eq!(lp_token.balance(&user), 10_000);
    assert_eq!(
        staking.query_staked(&user),
        StakedResponse {
            stakes: Vec::new(&env),
            total_stake: 0,
            last_reward_time: SIXTY_DAYS + 20 * ONE_DAY,
        }
    );

    // A follow-up withdraw_rewards must be a no-op (we already paid).
    let user_balance_before_second = reward_token.balance(&user);
    staking.withdraw_rewards(&user);
    assert_eq!(reward_token.balance(&user), user_balance_before_second);
}

#[test]
fn explicit_withdraw_then_unbond_does_not_double_pay() {
    // Backwards-compat invariant: the historically-documented
    // "withdraw_rewards-before-unbond" pattern must still work without
    // double-paying. With the fix in place, the unbond's internal settle
    // simply finds 0 pending and is a no-op.
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let manager = Address::generate(&env);

    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);
    let staking = deploy_staking_contract(
        &env,
        admin.clone(),
        &lp_token.address,
        &manager,
        &admin,
        &DEFAULT_COMPLEXITY,
    );

    lp_token.mint(&user, &10_000);
    reward_token.mint(&admin, &20_000);

    let staked = 1_000;
    staking.bond(&user, &staked);
    env.ledger().with_mut(|li| li.timestamp = SIXTY_DAYS);
    staking.create_distribution_flow(&admin, &reward_token.address);
    for _ in 0..20 {
        staking.distribute_rewards(&admin, &1_000, &reward_token.address);
        env.ledger().with_mut(|li| li.timestamp += ONE_DAY);
    }

    staking.withdraw_rewards(&user);
    assert_eq!(reward_token.balance(&user), 20_000);

    staking.unbond(&user, &staked, &0);

    // Crucial assertion: total = 20_000, not 40_000.
    assert_eq!(
        reward_token.balance(&user),
        20_000,
        "unbond must not double-pay rewards that were just claimed"
    );
}

#[test]
fn partial_unbond_settles_then_continues_to_earn() {
    // Two stakes, one is unbonded, the other remains. Settlement at unbond
    // must credit the full proportional reward across BOTH stakes (up to
    // now), and the remaining stake must keep earning on subsequent
    // distributions.
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let manager = Address::generate(&env);

    let lp_token = deploy_token_contract(&env, &admin);
    let reward_token = deploy_token_contract(&env, &admin);
    let staking = deploy_staking_contract(
        &env,
        admin.clone(),
        &lp_token.address,
        &manager,
        &admin,
        &DEFAULT_COMPLEXITY,
    );

    lp_token.mint(&user, &10_000);
    reward_token.mint(&admin, &100_000);

    // Two bonds at separate timestamps so they have distinct (stake,
    // stake_timestamp) keys.
    staking.bond(&user, &1_000);
    let first_stake_timestamp = 0u64;

    env.ledger().with_mut(|li| li.timestamp = SIXTY_DAYS);
    staking.bond(&user, &2_000); // matures at SIXTY_DAYS + 60d
    let second_stake_timestamp = SIXTY_DAYS;

    // Push to where both stakes are fully matured.
    env.ledger().with_mut(|li| li.timestamp = SIXTY_DAYS * 2);
    staking.create_distribution_flow(&admin, &reward_token.address);

    // 5 days of distributions before the partial unbond.
    for _ in 0..5 {
        staking.distribute_rewards(&admin, &1_000, &reward_token.address);
        env.ledger().with_mut(|li| li.timestamp += ONE_DAY);
    }

    // Unbond the FIRST stake (1_000). Pending rewards across both stakes
    // must settle into the user's account first.
    let user_balance_before = reward_token.balance(&user);
    staking.unbond(&user, &1_000, &first_stake_timestamp);
    let credited_at_unbond = reward_token.balance(&user) - user_balance_before;
    assert!(
        credited_at_unbond > 0,
        "partial unbond must credit accrued rewards on the remaining + removed stakes"
    );

    // The second stake remains and continues to earn. Another 5 days of
    // distributions then a withdraw_rewards call must credit additional
    // reward (computed only against the surviving stake of 2_000).
    for _ in 0..5 {
        staking.distribute_rewards(&admin, &1_000, &reward_token.address);
        env.ledger().with_mut(|li| li.timestamp += ONE_DAY);
    }
    let before_second_withdraw = reward_token.balance(&user);
    staking.withdraw_rewards(&user);
    let new_rewards = reward_token.balance(&user) - before_second_withdraw;
    assert!(
        new_rewards > 0,
        "the surviving stake must keep accruing after a partial unbond"
    );

    // The remaining staked principal must equal the second bond.
    let staked = staking.query_staked(&user);
    assert_eq!(staked.total_stake, 2_000);
    assert_eq!(staked.stakes.len(), 1);
    assert_eq!(staked.stakes.get(0).unwrap().stake, 2_000);
    assert_eq!(
        staked.stakes.get(0).unwrap().stake_timestamp,
        second_stake_timestamp
    );
}

#[test]
fn unbond_with_no_pending_rewards_succeeds() {
    // No distribution flow exists at all → settlement is a structural
    // no-op (the `get_distributions` loop has nothing to iterate).
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let manager = Address::generate(&env);

    let lp_token = deploy_token_contract(&env, &admin);
    let staking = deploy_staking_contract(
        &env,
        admin.clone(),
        &lp_token.address,
        &manager,
        &admin,
        &DEFAULT_COMPLEXITY,
    );

    lp_token.mint(&user, &5_000);
    staking.bond(&user, &1_000);
    env.ledger().with_mut(|li| li.timestamp = ONE_DAY);
    staking.unbond(&user, &1_000, &0);

    // User should hold all the LP tokens they had + the unbonded stake.
    assert_eq!(lp_token.balance(&user), 5_000);
    assert_eq!(
        staking.query_staked(&user),
        StakedResponse {
            stakes: Vec::new(&env),
            total_stake: 0,
            last_reward_time: ONE_DAY,
        }
    );
}
