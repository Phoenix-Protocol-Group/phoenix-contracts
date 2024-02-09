use crate::storage::Swap;
use crate::tests::setup::{
    deploy_and_initialize_factory, deploy_and_initialize_lp, deploy_and_mint_tokens,
    deploy_multihop_contract, deploy_token_contract,
};

use soroban_sdk::{testutils::Address as _, vec, Address, Env};

#[test]
fn simulate_swap_single_pool_no_fees() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);

    let token1 = deploy_and_mint_tokens(&env, &admin, 100_000_000i128);
    let token2 = deploy_and_mint_tokens(&env, &admin, 200_000_000i128);

    let factory_client = deploy_and_initialize_factory(&env, admin.clone());

    // 1:2 token ratio
    deploy_and_initialize_lp(
        &env,
        &factory_client,
        admin.clone(),
        token1.address.clone(),
        100_000_000,
        token2.address.clone(),
        200_000_000,
        None,
    );

    let multihop = deploy_multihop_contract(&env, admin.clone(), &factory_client.address);

    let operation = vec![
        &env,
        Swap {
            offer_asset: token1.address.clone(),
            ask_asset: token2.address.clone(),
            max_belief_price: None::<i64>,
        },
    ];

    // Offering 1k token1 should result in 2k token2
    let result = multihop.simulate_swap(&operation, &1_000);

    assert_eq!(result.ask_amount, 2_000i128);
    assert_eq!(result.total_commission_amount, 0i128);
    assert_eq!(result.spread_amount, vec![&env, 0i128]);

    // simulate reverse swap for exact results
    let reverse_simulated_swap = multihop.simulate_reverse_swap(&operation, &2_000i128);

    assert_eq!(reverse_simulated_swap.offer_amount, 1_000i128);
    assert_eq!(reverse_simulated_swap.total_commission_amount, 0i128);
    assert_eq!(reverse_simulated_swap.spread_amount, vec![&env, 0i128]);
}

#[test]
fn simulate_swap_three_equal_pools_no_fees() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);

    let token1 = deploy_and_mint_tokens(&env, &admin, 10_000_000i128);
    let token2 = deploy_and_mint_tokens(&env, &admin, 10_000_000i128);
    let token3 = deploy_and_mint_tokens(&env, &admin, 10_000_000i128);
    let token4 = deploy_and_mint_tokens(&env, &admin, 10_000_000i128);

    let factory_client = deploy_and_initialize_factory(&env, admin.clone());

    deploy_and_initialize_lp(
        &env,
        &factory_client,
        admin.clone(),
        token1.address.clone(),
        1_000_000,
        token2.address.clone(),
        1_000_000,
        None,
    );
    deploy_and_initialize_lp(
        &env,
        &factory_client,
        admin.clone(),
        token2.address.clone(),
        1_000_000,
        token3.address.clone(),
        1_000_000,
        None,
    );
    deploy_and_initialize_lp(
        &env,
        &factory_client,
        admin.clone(),
        token3.address.clone(),
        1_000_000,
        token4.address.clone(),
        1_000_000,
        None,
    );

    let multihop = deploy_multihop_contract(&env, admin.clone(), &factory_client.address);

    // Very low amount will result in equal 1:1 swaps
    let simulated_swap = multihop.simulate_swap(
        &vec![
            &env,
            Swap {
                offer_asset: token1.address.clone(),
                ask_asset: token2.address.clone(),
                max_belief_price: None::<i64>,
            },
            Swap {
                offer_asset: token2.address.clone(),
                ask_asset: token3.address.clone(),
                max_belief_price: None::<i64>,
            },
            Swap {
                offer_asset: token3.address.clone(),
                ask_asset: token4.address.clone(),
                max_belief_price: None::<i64>,
            },
        ],
        &50i128,
    );

    assert_eq!(simulated_swap.ask_amount, 50i128);
    assert_eq!(simulated_swap.total_commission_amount, 0i128);
    assert_eq!(
        simulated_swap.spread_amount,
        vec![&env, 0i128, 0i128, 0i128]
    );

    // simulate reverse swap for exact results
    let reverse_simulated_swap = multihop.simulate_reverse_swap(
        &vec![
            &env,
            Swap {
                offer_asset: token3.address.clone(),
                ask_asset: token4.address.clone(),
                max_belief_price: None::<i64>,
            },
            Swap {
                offer_asset: token2.address.clone(),
                ask_asset: token3.address.clone(),
                max_belief_price: None::<i64>,
            },
            Swap {
                offer_asset: token1.address.clone(),
                ask_asset: token2.address.clone(),
                max_belief_price: None::<i64>,
            },
        ],
        &50i128,
    );

    assert_eq!(reverse_simulated_swap.offer_amount, 50i128);
    assert_eq!(reverse_simulated_swap.total_commission_amount, 0i128);
    assert_eq!(
        reverse_simulated_swap.spread_amount,
        vec![&env, 0i128, 0i128, 0i128]
    );
}

#[test]
fn simulate_swap_single_pool_with_fees() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);

    let token1 = deploy_and_mint_tokens(&env, &admin, 1_001_000i128);
    let token2 = deploy_and_mint_tokens(&env, &admin, 1_001_000i128);

    let factory_client = deploy_and_initialize_factory(&env, admin.clone());

    deploy_and_initialize_lp(
        &env,
        &factory_client,
        admin.clone(),
        token1.address.clone(),
        1_000_000,
        token2.address.clone(),
        1_000_000,
        Some(2000),
    );

    // 4. swap with multihop
    let multihop = deploy_multihop_contract(&env, admin.clone(), &factory_client.address);

    let operation = vec![
        &env,
        Swap {
            offer_asset: token1.address.clone(),
            ask_asset: token2.address.clone(),
            max_belief_price: None::<i64>,
        },
    ];

    let simulated_swap = multihop.simulate_swap(&operation, &300i128);

    // 1000 tokens initially
    // swap 300 from token1 to token2 with 2000 bps (20%)
    // tokens2 will be 240
    assert_eq!(simulated_swap.ask_amount, 240i128);
    assert_eq!(simulated_swap.total_commission_amount, 60i128);
    assert_eq!(simulated_swap.spread_amount, vec![&env, 0i128]);

    // simulate reverse swap returns same result
    let reverse_simulated_swap = multihop.simulate_reverse_swap(&operation, &240i128);

    assert_eq!(reverse_simulated_swap.offer_amount, 300i128);
    assert_eq!(reverse_simulated_swap.total_commission_amount, 60i128);
    assert_eq!(reverse_simulated_swap.spread_amount, vec![&env, 0i128]);
}

#[test]
fn simulate_swap_three_different_pools_no_fees() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);

    let token1 = deploy_and_mint_tokens(&env, &admin, 10_000_000i128);
    let token2 = deploy_and_mint_tokens(&env, &admin, 10_000_000i128);
    let token3 = deploy_and_mint_tokens(&env, &admin, 10_000_000i128);
    let token4 = deploy_and_mint_tokens(&env, &admin, 10_000_000i128);

    let factory_client = deploy_and_initialize_factory(&env, admin.clone());

    deploy_and_initialize_lp(
        &env,
        &factory_client,
        admin.clone(),
        token1.address.clone(),
        1_000_000,
        token2.address.clone(),
        1_000_000,
        None,
    );
    deploy_and_initialize_lp(
        &env,
        &factory_client,
        admin.clone(),
        token2.address.clone(),
        2_000_000,
        token3.address.clone(),
        2_000_000,
        None,
    );
    deploy_and_initialize_lp(
        &env,
        &factory_client,
        admin.clone(),
        token3.address.clone(),
        3_000_000,
        token4.address.clone(),
        3_000_000,
        None,
    );

    let multihop = deploy_multihop_contract(&env, admin.clone(), &factory_client.address);

    let simulated_swap = multihop.simulate_swap(
        &vec![
            &env,
            Swap {
                offer_asset: token1.address.clone(),
                ask_asset: token2.address.clone(),
                max_belief_price: None::<i64>,
            },
            Swap {
                offer_asset: token2.address.clone(),
                ask_asset: token3.address.clone(),
                max_belief_price: None::<i64>,
            },
            Swap {
                offer_asset: token3.address.clone(),
                ask_asset: token4.address.clone(),
                max_belief_price: None::<i64>,
            },
        ],
        &5_000i128,
    );

    // constant product formula starts to with which amoutns such as 5k
    assert_eq!(simulated_swap.ask_amount, 4_956i128);
    assert_eq!(simulated_swap.total_commission_amount, 0i128);
    assert_eq!(
        simulated_swap.spread_amount,
        vec![&env, 24i128, 12i128, 8i128]
    );

    // simulate reverse swap returns same result
    let reverse_simulated_swap = multihop.simulate_reverse_swap(
        &vec![
            &env,
            Swap {
                offer_asset: token3.address.clone(),
                ask_asset: token4.address.clone(),
                max_belief_price: None::<i64>,
            },
            Swap {
                offer_asset: token2.address.clone(),
                ask_asset: token3.address.clone(),
                max_belief_price: None::<i64>,
            },
            Swap {
                offer_asset: token1.address.clone(),
                ask_asset: token2.address.clone(),
                max_belief_price: None::<i64>,
            },
        ],
        &4_956i128,
    );

    assert_eq!(reverse_simulated_swap.offer_amount, 5_000i128);
    assert_eq!(reverse_simulated_swap.total_commission_amount, 0i128);
    assert_eq!(
        reverse_simulated_swap.spread_amount,
        vec![&env, 8i128, 12i128, 24i128]
    );
}

#[test]
fn simulate_swap_three_different_pools_with_fees() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);

    let token1 = deploy_and_mint_tokens(&env, &admin, 10_000_000i128);
    let token2 = deploy_and_mint_tokens(&env, &admin, 10_000_000i128);
    let token3 = deploy_and_mint_tokens(&env, &admin, 10_000_000i128);
    let token4 = deploy_and_mint_tokens(&env, &admin, 10_000_000i128);

    let factory_client = deploy_and_initialize_factory(&env, admin.clone());

    let fees = Some(1_000); // 1000bps == 10%
    deploy_and_initialize_lp(
        &env,
        &factory_client,
        admin.clone(),
        token1.address.clone(),
        1_000_000,
        token2.address.clone(),
        2_000_000,
        fees,
    );
    deploy_and_initialize_lp(
        &env,
        &factory_client,
        admin.clone(),
        token2.address.clone(),
        1_000_000,
        token3.address.clone(),
        3_000_000,
        fees,
    );
    deploy_and_initialize_lp(
        &env,
        &factory_client,
        admin.clone(),
        token3.address.clone(),
        1_000_000,
        token4.address.clone(),
        5_000_000,
        fees,
    );

    let multihop = deploy_multihop_contract(&env, admin.clone(), &factory_client.address);

    let simulated_swap = multihop.simulate_swap(
        &vec![
            &env,
            Swap {
                offer_asset: token1.address.clone(),
                ask_asset: token2.address.clone(),
                max_belief_price: None::<i64>,
            },
            Swap {
                offer_asset: token2.address.clone(),
                ask_asset: token3.address.clone(),
                max_belief_price: None::<i64>,
            },
            Swap {
                offer_asset: token3.address.clone(),
                ask_asset: token4.address.clone(),
                max_belief_price: None::<i64>,
            },
        ],
        &10_000i128,
    );

    // cp = offer_pool * ask_pool
    // return_amount = ask_pool - (cp / (offer_pool + offer_amount))
    // commission_amount = return_amount * commission_rate

    // we start swapping 10_000 tokens

    // token1 => token2
    // cp = 2_000_000_000_000
    // return_amount = 2_000_000 - (2 * 10^12 / (1_000_000 + 10_000)) = 19_802
    // commission_amount = 1_980.2
    // ask_amount = 19_802 - 1_980 = 17_822

    // token2 => token3
    // cp = 3_000_000_000_000
    // return_amount = 3_000_000 - (3 * 10^12 / (1_000_000 + 17_822)) = 52_529.82
    // commission_amount = 5_252.9
    // ask_amount = 52_529 - 5_252 = 47_277

    // token3 => token4
    // cp = 5_000_000_000_000
    // return_amount = 5_000_000 - (5 * 10^12 / (1_000_000 + 47_277)) = 225_713.93
    // commission_amount = 22_571.3
    // ask_amount = 225_714 - 22_571 = 203_143
    assert_eq!(simulated_swap.ask_amount, 203_143i128);
    // total_commission_amount = 1_980 + 5_253 + 22_571 = 29_804
    assert_eq!(simulated_swap.total_commission_amount, 29_804i128);
    assert_eq!(
        simulated_swap.spread_amount,
        vec![&env, 198i128, 936i128, 10671i128]
    );

    // simulate reverse swap returns same result
    let reverse_simulated_swap = multihop.simulate_reverse_swap(
        &vec![
            &env,
            Swap {
                offer_asset: token3.address.clone(),
                ask_asset: token4.address.clone(),
                max_belief_price: None::<i64>,
            },
            Swap {
                offer_asset: token2.address.clone(),
                ask_asset: token3.address.clone(),
                max_belief_price: None::<i64>,
            },
            Swap {
                offer_asset: token1.address.clone(),
                ask_asset: token2.address.clone(),
                max_belief_price: None::<i64>,
            },
        ],
        &203_143i128,
    );

    // one difference due to rounding
    assert_eq!(reverse_simulated_swap.offer_amount, 9_999i128);
    assert_eq!(reverse_simulated_swap.total_commission_amount, 29_803i128);
    assert_eq!(
        reverse_simulated_swap.spread_amount,
        vec![&env, 10671i128, 934i128, 197i128]
    );
}

#[test]
#[should_panic(expected = "Multihop: Simulate swap: operations empty")]
fn query_simulate_swap_panics_with_no_operations() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let factory = Address::generate(&env);
    let recipient = Address::generate(&env);

    let token = deploy_token_contract(&env, &admin);
    token.mint(&recipient, &50i128);

    let multihop = deploy_multihop_contract(&env, admin, &factory);

    let swap_vec = vec![&env];

    multihop.simulate_swap(&swap_vec, &50i128);
}

#[test]
#[should_panic(expected = "Multihop: Simulate reverse swap: operations empty")]
fn query_simulate_reverse_swap_panics_with_no_operations() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let factory = Address::generate(&env);
    let recipient = Address::generate(&env);

    let token = deploy_token_contract(&env, &admin);
    token.mint(&recipient, &50i128);

    let multihop = deploy_multihop_contract(&env, admin, &factory);

    let swap_vec = vec![&env];

    multihop.simulate_reverse_swap(&swap_vec, &50i128);
}
