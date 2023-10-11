use crate::storage::Swap;
use crate::tests::setup::factory::{LiquidityPoolInitInfo, StakeInitInfo, TokenInitInfo};
use crate::tests::setup::{
    deploy_factory_contract, deploy_multihop_contract, deploy_token_contract, factory,
    install_lp_contract, install_stake_wasm, install_token_wasm, lp_contract, token_contract,
};
use soroban_sdk::{testutils::Address as _, vec, Address, Env};

#[test]
fn swap_three_equal_pools_no_fees() {
    let env = Env::default();

    let admin = Address::random(&env);

    let tokens = generate_minted_tokens::<4>(&env, admin.clone(), 4, false);

    // 1. deploy factory

    let factory_client = get_factory(&env, admin.clone());

    // 2. create liquidity pool from factory
    let lp_wasm_hash = install_lp_contract(&env);

    let first_token_init_info = TokenInitInfo {
        token_wasm_hash: install_token_wasm(&env),
        token_a: tokens[0].address.clone(),
        token_b: tokens[1].address.clone(),
    };
    let first_stake_init_info = StakeInitInfo {
        stake_wasm_hash: install_stake_wasm(&env),
        min_bond: 10i128,
        max_distributions: 10u32,
        min_reward: 5i128,
    };

    let first_lp_init_info = LiquidityPoolInitInfo {
        admin: admin.clone(),
        fee_recipient: admin.clone(),
        lp_wasm_hash: lp_wasm_hash.clone(),
        max_allowed_slippage_bps: 5000,
        max_allowed_spread_bps: 500,
        share_token_decimals: 7,
        swap_fee_bps: 0,
        token_init_info: first_token_init_info.clone(),
        stake_init_info: first_stake_init_info,
    };

    let second_token_init_info = TokenInitInfo {
        token_wasm_hash: install_token_wasm(&env),
        token_a: tokens[1].address.clone(),
        token_b: tokens[2].address.clone(),
    };
    let second_stake_init_info = StakeInitInfo {
        stake_wasm_hash: install_stake_wasm(&env),
        min_bond: 5i128,
        max_distributions: 5u32,
        min_reward: 2i128,
    };

    let second_lp_init_info = LiquidityPoolInitInfo {
        admin: admin.clone(),
        fee_recipient: admin.clone(),
        lp_wasm_hash: lp_wasm_hash.clone(),
        max_allowed_slippage_bps: 4_000,
        max_allowed_spread_bps: 400,
        share_token_decimals: 6,
        swap_fee_bps: 0,
        token_init_info: second_token_init_info,
        stake_init_info: second_stake_init_info,
    };

    let third_token_init_info = TokenInitInfo {
        token_wasm_hash: install_token_wasm(&env),
        token_a: tokens[2].address.clone(),
        token_b: tokens[3].address.clone(),
    };
    let third_stake_init_info = StakeInitInfo {
        stake_wasm_hash: install_stake_wasm(&env),
        min_bond: 6i128,
        max_distributions: 6u32,
        min_reward: 3i128,
    };

    let third_lp_init_info = LiquidityPoolInitInfo {
        admin: admin.clone(),
        fee_recipient: admin.clone(),
        lp_wasm_hash,
        max_allowed_slippage_bps: 4_000,
        max_allowed_spread_bps: 400,
        share_token_decimals: 6,
        swap_fee_bps: 0,
        token_init_info: third_token_init_info,
        stake_init_info: third_stake_init_info,
    };

    let first_lp = factory_client.create_liquidity_pool(&first_lp_init_info);
    let second_lp = factory_client.create_liquidity_pool(&second_lp_init_info);
    let third_lp = factory_client.create_liquidity_pool(&third_lp_init_info);

    // 3. provide liquidity for each one of the liquidity pools
    for lp in factory_client.query_pools() {
        let lp_client = lp_contract::Client::new(&env, &lp);
        lp_client.provide_liquidity(
            &admin.clone(),
            &Some(1_000_000i128),
            &None,
            &Some(1_000_000i128),
            &None,
            &None::<i64>,
        );
    }

    // 4. swap with multihop
    let multihop = deploy_multihop_contract(&env, admin, &factory_client.address);
    let recipient = Address::random(&env);
    tokens[0].mint(&recipient, &50i128);
    assert_eq!(tokens[0].balance(&recipient), 50i128);
    assert_eq!(tokens[3].balance(&recipient), 0i128);

    let swap1 = Swap {
        offer_asset: tokens[0].address.clone(),
        ask_asset: tokens[1].address.clone(),
    };
    let swap2 = Swap {
        offer_asset: tokens[1].address.clone(),
        ask_asset: tokens[2].address.clone(),
    };
    let swap3 = Swap {
        offer_asset: tokens[2].address.clone(),
        ask_asset: tokens[3].address.clone(),
    };

    let operations = vec![&env, swap1, swap2, swap3];

    multihop.swap(&recipient, &operations, &50i128);

    // 5. check if it goes according to plan
    assert_eq!(tokens[0].balance(&recipient), 0i128);
    assert_eq!(tokens[3].balance(&recipient), 50i128);
}

#[test]
fn swap_single_pool_no_fees() {
    let env = Env::default();
    let admin = Address::random(&env);

    env.mock_all_auths();
    env.budget().reset_unlimited();

    let tokens = generate_minted_tokens(&env, admin.clone(), 2, false);

    // 1. deploy factory
    let factory_client = get_factory(&env, admin.clone());

    // 2. create liquidity pool from factory
    let lp_wasm_hash = install_lp_contract(&env);

    let first_token_init_info = TokenInitInfo {
        token_wasm_hash: install_token_wasm(&env),
        token_a: tokens[0].address.clone(),
        token_b: tokens[1].address.clone(),
    };
    let first_stake_init_info = StakeInitInfo {
        stake_wasm_hash: install_stake_wasm(&env),
        min_bond: 10i128,
        max_distributions: 10u32,
        min_reward: 5i128,
    };

    let first_lp_init_info = LiquidityPoolInitInfo {
        admin: admin.clone(),
        fee_recipient: admin.clone(),
        lp_wasm_hash: lp_wasm_hash.clone(),
        max_allowed_slippage_bps: 5000,
        max_allowed_spread_bps: 500,
        share_token_decimals: 7,
        swap_fee_bps: 0,
        token_init_info: first_token_init_info.clone(),
        stake_init_info: first_stake_init_info,
    };

    factory_client.create_liquidity_pool(&first_lp_init_info);

    // 3. provide liquidity for each one of the liquidity pools
    for lp in factory_client.query_pools() {
        let lp_client = lp_contract::Client::new(&env, &lp);
        lp_client.provide_liquidity(
            &admin.clone(),
            &Some(1_000_000i128),
            &None,
            &Some(1_000_000i128),
            &None,
            &None::<i64>,
        );
    }

    // 4. swap with multihop
    let multihop = deploy_multihop_contract(&env, admin, &factory_client.address);
    let recipient = Address::random(&env);
    tokens[0].mint(&recipient, &50i128);
    assert_eq!(tokens[0].balance(&recipient), 50i128);
    assert_eq!(tokens[1].balance(&recipient), 0i128);

    let swap1 = Swap {
        offer_asset: tokens[0].address.clone(),
        ask_asset: tokens[1].address.clone(),
    };

    let operations = vec![&env, swap1];

    multihop.swap(&recipient, &operations, &50i128);

    // 5. check if it goes according to plan
    assert_eq!(tokens[0].balance(&recipient), 0i128);
    assert_eq!(tokens[1].balance(&recipient), 50i128);
}

#[test]
fn swap_single_pool_with_fees() {
    let env = Env::default();
    let admin = Address::random(&env);

    env.mock_all_auths();
    env.budget().reset_unlimited();

    let tokens = generate_minted_tokens(&env, admin.clone(), 2, false);

    // 1. deploy factory
    let factory_client = get_factory(&env, admin.clone());

    // 2. create liquidity pool from factory
    let lp_wasm_hash = install_lp_contract(&env);

    let first_token_init_info = TokenInitInfo {
        token_wasm_hash: install_token_wasm(&env),
        token_a: tokens[0].address.clone(),
        token_b: tokens[1].address.clone(),
    };
    let first_stake_init_info = StakeInitInfo {
        stake_wasm_hash: install_stake_wasm(&env),
        min_bond: 10i128,
        max_distributions: 10u32,
        min_reward: 5i128,
    };

    let first_lp_init_info = LiquidityPoolInitInfo {
        admin: admin.clone(),
        fee_recipient: admin.clone(),
        lp_wasm_hash: lp_wasm_hash.clone(),
        max_allowed_slippage_bps: 5000,
        max_allowed_spread_bps: 500,
        share_token_decimals: 7,
        swap_fee_bps: 1500,
        token_init_info: first_token_init_info.clone(),
        stake_init_info: first_stake_init_info,
    };

    factory_client.create_liquidity_pool(&first_lp_init_info);

    // 3. provide liquidity for each one of the liquidity pools
    for lp in factory_client.query_pools() {
        let lp_client = lp_contract::Client::new(&env, &lp);
        lp_client.provide_liquidity(
            &admin.clone(),
            &Some(1_000_000i128),
            &None,
            &Some(1_000_000i128),
            &None,
            &None::<i64>,
        );
    }

    // 4. swap with multihop
    let multihop = deploy_multihop_contract(&env, admin, &factory_client.address);
    let recipient = Address::random(&env);
    tokens[0].mint(&recipient, &50i128);
    assert_eq!(tokens[0].balance(&recipient), 50i128);
    assert_eq!(tokens[1].balance(&recipient), 0i128);

    let swap1 = Swap {
        offer_asset: tokens[0].address.clone(),
        ask_asset: tokens[1].address.clone(),
    };

    let operations = vec![&env, swap1];

    multihop.swap(&recipient, &operations, &50i128);

    // 5. check if it goes according to plan
    assert_eq!(tokens[0].balance(&recipient), 0i128);
    assert_eq!(tokens[1].balance(&recipient), 43i128);
}

#[test]
fn swap_three_different_pools_no_fees() {
    let env = Env::default();

    let admin = Address::random(&env);

    env.mock_all_auths();
    env.budget().reset_unlimited();

    let tokens = generate_minted_tokens(&env, admin.clone(), 4, true);

    // 1. deploy factory
    let factory_client = get_factory(&env, admin.clone());

    // 2. create liquidity pool from factory
    let lp_wasm_hash = install_lp_contract(&env);

    let first_token_init_info = TokenInitInfo {
        token_wasm_hash: install_token_wasm(&env),
        token_a: tokens[0].address.clone(),
        token_b: tokens[1].address.clone(),
    };
    let first_stake_init_info = StakeInitInfo {
        stake_wasm_hash: install_stake_wasm(&env),
        min_bond: 10i128,
        max_distributions: 10u32,
        min_reward: 5i128,
    };

    let first_lp_init_info = LiquidityPoolInitInfo {
        admin: admin.clone(),
        fee_recipient: admin.clone(),
        lp_wasm_hash: lp_wasm_hash.clone(),
        max_allowed_slippage_bps: 5000,
        max_allowed_spread_bps: 500,
        share_token_decimals: 7,
        swap_fee_bps: 0,
        token_init_info: first_token_init_info.clone(),
        stake_init_info: first_stake_init_info,
    };

    let second_token_init_info = TokenInitInfo {
        token_wasm_hash: install_token_wasm(&env),
        token_a: tokens[1].address.clone(),
        token_b: tokens[2].address.clone(),
    };
    let second_stake_init_info = StakeInitInfo {
        stake_wasm_hash: install_stake_wasm(&env),
        min_bond: 5i128,
        max_distributions: 5u32,
        min_reward: 2i128,
    };

    let second_lp_init_info = LiquidityPoolInitInfo {
        admin: admin.clone(),
        fee_recipient: admin.clone(),
        lp_wasm_hash: lp_wasm_hash.clone(),
        max_allowed_slippage_bps: 4_000,
        max_allowed_spread_bps: 400,
        share_token_decimals: 6,
        swap_fee_bps: 0,
        token_init_info: second_token_init_info,
        stake_init_info: second_stake_init_info,
    };

    let third_token_init_info = TokenInitInfo {
        token_wasm_hash: install_token_wasm(&env),
        token_a: tokens[2].address.clone(),
        token_b: tokens[3].address.clone(),
    };
    let third_stake_init_info = StakeInitInfo {
        stake_wasm_hash: install_stake_wasm(&env),
        min_bond: 6i128,
        max_distributions: 6u32,
        min_reward: 3i128,
    };

    let third_lp_init_info = LiquidityPoolInitInfo {
        admin: admin.clone(),
        fee_recipient: admin.clone(),
        lp_wasm_hash,
        max_allowed_slippage_bps: 4_000,
        max_allowed_spread_bps: 400,
        share_token_decimals: 6,
        swap_fee_bps: 0,
        token_init_info: third_token_init_info,
        stake_init_info: third_stake_init_info,
    };

    factory_client.create_liquidity_pool(&first_lp_init_info);
    factory_client.create_liquidity_pool(&second_lp_init_info);
    factory_client.create_liquidity_pool(&third_lp_init_info);

    // 3. provide liquidity for each one of the liquidity pools
    for lp in factory_client.query_pools() {
        let lp_client = lp_contract::Client::new(&env, &lp);
        lp_client.provide_liquidity(
            &admin.clone(),
            &Some(1_000_000i128),
            &None,
            &Some(2_000_000i128),
            &None,
            &None::<i64>,
        );
    }

    // 4. swap with multihop
    let multihop = deploy_multihop_contract(&env, admin, &factory_client.address);
    let recipient = Address::random(&env);
    tokens[0].mint(&recipient, &50i128);
    assert_eq!(tokens[0].balance(&recipient), 50i128);
    assert_eq!(tokens[3].balance(&recipient), 0i128);

    let swap1 = Swap {
        offer_asset: tokens[0].address.clone(),
        ask_asset: tokens[1].address.clone(),
    };
    let swap2 = Swap {
        offer_asset: tokens[1].address.clone(),
        ask_asset: tokens[2].address.clone(),
    };
    let swap3 = Swap {
        offer_asset: tokens[2].address.clone(),
        ask_asset: tokens[3].address.clone(),
    };

    let operations = vec![&env, swap1, swap2, swap3];

    multihop.swap(&recipient, &operations, &50i128);

    // 5. check if it goes according to plan
    assert_eq!(tokens[0].balance(&recipient), 0i128);
    assert_eq!(tokens[3].balance(&recipient), 400i128);
}

#[test]
fn swap_three_different_pools_with_fees() {
    let env = Env::default();

    let admin = Address::random(&env);

    env.mock_all_auths();
    env.budget().reset_unlimited();

    let tokens = generate_minted_tokens(&env, admin.clone(), 4, true);

    // 1. deploy factory
    let factory_client = get_factory(&env, admin.clone());

    // 2. create liquidity pool from factory
    let lp_wasm_hash = install_lp_contract(&env);

    let first_token_init_info = TokenInitInfo {
        token_wasm_hash: install_token_wasm(&env),
        token_a: tokens[0].address.clone(),
        token_b: tokens[1].address.clone(),
    };
    let first_stake_init_info = StakeInitInfo {
        stake_wasm_hash: install_stake_wasm(&env),
        min_bond: 10i128,
        max_distributions: 10u32,
        min_reward: 5i128,
    };

    let first_lp_init_info = LiquidityPoolInitInfo {
        admin: admin.clone(),
        fee_recipient: admin.clone(),
        lp_wasm_hash: lp_wasm_hash.clone(),
        max_allowed_slippage_bps: 5000,
        max_allowed_spread_bps: 500,
        share_token_decimals: 7,
        swap_fee_bps: 1500,
        token_init_info: first_token_init_info.clone(),
        stake_init_info: first_stake_init_info,
    };

    let second_token_init_info = TokenInitInfo {
        token_wasm_hash: install_token_wasm(&env),
        token_a: tokens[1].address.clone(),
        token_b: tokens[2].address.clone(),
    };
    let second_stake_init_info = StakeInitInfo {
        stake_wasm_hash: install_stake_wasm(&env),
        min_bond: 5i128,
        max_distributions: 5u32,
        min_reward: 2i128,
    };

    let second_lp_init_info = LiquidityPoolInitInfo {
        admin: admin.clone(),
        fee_recipient: admin.clone(),
        lp_wasm_hash: lp_wasm_hash.clone(),
        max_allowed_slippage_bps: 4_000,
        max_allowed_spread_bps: 400,
        share_token_decimals: 6,
        swap_fee_bps: 1500,
        token_init_info: second_token_init_info,
        stake_init_info: second_stake_init_info,
    };

    let third_token_init_info = TokenInitInfo {
        token_wasm_hash: install_token_wasm(&env),
        token_a: tokens[2].address.clone(),
        token_b: tokens[3].address.clone(),
    };
    let third_stake_init_info = StakeInitInfo {
        stake_wasm_hash: install_stake_wasm(&env),
        min_bond: 6i128,
        max_distributions: 6u32,
        min_reward: 3i128,
    };

    let third_lp_init_info = LiquidityPoolInitInfo {
        admin: admin.clone(),
        fee_recipient: admin.clone(),
        lp_wasm_hash,
        max_allowed_slippage_bps: 4_000,
        max_allowed_spread_bps: 400,
        share_token_decimals: 6,
        swap_fee_bps: 1500,
        token_init_info: third_token_init_info,
        stake_init_info: third_stake_init_info,
    };

    factory_client.create_liquidity_pool(&first_lp_init_info);
    factory_client.create_liquidity_pool(&second_lp_init_info);
    factory_client.create_liquidity_pool(&third_lp_init_info);

    // 3. provide liquidity for each one of the liquidity pools
    for lp in factory_client.query_pools() {
        let lp_client = lp_contract::Client::new(&env, &lp);
        lp_client.provide_liquidity(
            &admin.clone(),
            &Some(1_000_000i128),
            &None,
            &Some(2_000_000i128),
            &None,
            &None::<i64>,
        );
    }

    // 4. swap with multihop
    let multihop = deploy_multihop_contract(&env, admin, &factory_client.address);
    let recipient = Address::random(&env);
    tokens[0].mint(&recipient, &50i128);
    assert_eq!(tokens[0].balance(&recipient), 50i128);
    assert_eq!(tokens[3].balance(&recipient), 0i128);

    let swap1 = Swap {
        offer_asset: tokens[0].address.clone(),
        ask_asset: tokens[1].address.clone(),
    };
    let swap2 = Swap {
        offer_asset: tokens[1].address.clone(),
        ask_asset: tokens[2].address.clone(),
    };
    let swap3 = Swap {
        offer_asset: tokens[2].address.clone(),
        ask_asset: tokens[3].address.clone(),
    };

    let operations = vec![&env, swap1, swap2, swap3];

    multihop.swap(&recipient, &operations, &50i128);

    // 5. check if it goes according to plan
    assert_eq!(tokens[0].balance(&recipient), 0i128);
    assert_eq!(tokens[3].balance(&recipient), 247i128);
}

#[test]
#[should_panic(expected = "Multihop: Swap: Operations empty")]
fn swap_panics_with_no_operations() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::random(&env);
    let factory = Address::random(&env);

    let recipient = Address::random(&env);

    let multihop = deploy_multihop_contract(&env, admin, &factory);

    let swap_vec = vec![&env];

    multihop.swap(&recipient, &swap_vec, &50i128);
}

fn generate_minted_tokens<'a, const LEN: usize>(
    env: &Env,
    admin: Address,
    number_of_tokens: i32,
    different_token_ratio: bool,
) -> [token_contract::Client<'a>; LEN] {
    let mut tokens = [token_contract::Client, LEN];
    for i in 0..number_of_tokens {
        let token = deploy_token_contract(&env, &admin);
        tokens[i] = token
    }

    tokens.sort_by(|a, b| a.address.cmp(&b.address));

    match different_token_ratio {
        true => {
            let mut i = 1i128;
            tokens.iter().for_each(|mut t| {
                t.mint(&admin, &(i * 10_000_000i128));
                i += 1;
            });
        }
        false => {
            tokens.iter().for_each(|t| t.mint(&admin, &10_000_000i128));
        }
    }

    tokens
}

fn get_factory(env: &Env, admin: Address) -> factory::Client {
    let factory_addr = deploy_factory_contract(&env, admin.clone());
    let factory_client = factory::Client::new(&env, &factory_addr);

    factory_client.initialize(&admin.clone());
    factory_client
}
