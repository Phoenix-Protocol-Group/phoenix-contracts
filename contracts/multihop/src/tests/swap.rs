use crate::error::ContractError;
use crate::storage::Swap;
use crate::tests::setup::factory::{LiquidityPoolInitInfo, StakeInitInfo, TokenInitInfo};
use crate::tests::setup::{
    deploy_factory_contract, deploy_multihop_contract, deploy_token_contract, factory,
    install_lp_contract, install_stake_wasm, install_token_wasm, lp_contract, token_contract,
};

use soroban_sdk::arbitrary::std;
use soroban_sdk::{testutils::Address as _, vec, Address, Env};

#[test]
fn swap_three_equal_pools_no_fees() {
    let env = Env::default();

    let admin = Address::random(&env);

    env.mock_all_auths();
    env.budget().reset_unlimited();

    let token1 = deploy_and_mint_tokens(&env, &admin, 10_000_000i128);
    let token2 = deploy_and_mint_tokens(&env, &admin, 10_000_000i128);
    let token3 = deploy_and_mint_tokens(&env, &admin, 10_000_000i128);
    let token4 = deploy_and_mint_tokens(&env, &admin, 10_000_000i128);

    // 1. deploy factory
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

    // 4. swap with multihop
    let multihop = deploy_multihop_contract(&env, admin.clone(), &factory_client.address);
    let recipient = Address::random(&env);
    token1.mint(&recipient, &50i128);
    assert_eq!(token1.balance(&recipient), 50i128);
    assert_eq!(token4.balance(&recipient), 0i128);

    let swap1 = Swap {
        offer_asset: token1.address.clone(),
        ask_asset: token2.address.clone(),
    };
    let swap2 = Swap {
        offer_asset: token2.address.clone(),
        ask_asset: token3.address.clone(),
    };
    let swap3 = Swap {
        offer_asset: token3.address.clone(),
        ask_asset: token4.address.clone(),
    };

    let operations = vec![&env, swap1, swap2, swap3];

    multihop.swap(&recipient, &operations, &50i128);

    // 5. check if it goes according to plan
    assert_eq!(token1.balance(&recipient), 0i128);
    assert_eq!(token4.balance(&recipient), 50i128);
}

#[test]
fn swap_single_pool_no_fees() {
    let env = Env::default();
    let admin = Address::random(&env);

    env.mock_all_auths();
    env.budget().reset_unlimited();

    let token1 = deploy_and_mint_tokens(&env, &admin, 1_001_000i128);
    let token2 = deploy_and_mint_tokens(&env, &admin, 1_001_000i128);

    // 1. deploy factory
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

    // 4. swap with multihop
    let multihop = deploy_multihop_contract(&env, admin.clone(), &factory_client.address);
    let recipient = Address::random(&env);
    token1.mint(&recipient, &50i128); // mints 50 token0 to recipient
    assert_eq!(token1.balance(&recipient), 50i128);
    assert_eq!(token2.balance(&recipient), 0i128);

    let swap1 = Swap {
        offer_asset: token1.address.clone(),
        ask_asset: token2.address.clone(),
    };

    let operations = vec![&env, swap1];

    multihop.swap(&recipient, &operations, &1);

    // 5. check if it goes according to plan
    assert_eq!(token1.balance(&recipient), 49i128); // -1 token0
    assert_eq!(token2.balance(&recipient), 1i128); // +1 token1
}

#[test]
fn swap_single_pool_with_fees() {
    let env = Env::default();
    let admin = Address::random(&env);

    env.mock_all_auths();
    env.budget().reset_unlimited();

    let token1 = deploy_and_mint_tokens(&env, &admin, 1_001_000i128);
    let token2 = deploy_and_mint_tokens(&env, &admin, 1_001_000i128);

    // 1. deploy factory
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
    let recipient = Address::random(&env);
    token1.mint(&recipient, &1000i128);
    assert_eq!(token1.balance(&recipient), 1000i128);
    assert_eq!(token2.balance(&recipient), 0i128);

    let swap1 = Swap {
        offer_asset: token1.address.clone(),
        ask_asset: token2.address.clone(),
    };

    let operations = vec![&env, swap1];

    multihop.swap(&recipient, &operations, &300i128);

    // 5. check if it goes according to plan
    // 1000 tokens initially
    // swap 300 from token0 to token1 with 2000 bps (20%)
    // tokens1 will be 240
    assert_eq!(token1.balance(&recipient), 700i128);
    assert_eq!(token2.balance(&recipient), 240i128);
}

#[test]
fn swap_three_different_pools_no_fees() {
    let env = Env::default();

    let admin = Address::random(&env);

    env.mock_all_auths();
    env.budget().reset_unlimited();

    let token1 = deploy_and_mint_tokens(&env, &admin, 10_000_000i128);
    let token2 = deploy_and_mint_tokens(&env, &admin, 10_000_000i128);
    let token3 = deploy_and_mint_tokens(&env, &admin, 10_000_000i128);
    let token4 = deploy_and_mint_tokens(&env, &admin, 10_000_000i128);

    // 1. deploy factory
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

    // 4. swap with multihop
    let multihop = deploy_multihop_contract(&env, admin.clone(), &factory_client.address);
    let recipient = Address::random(&env);
    token1.mint(&recipient, &5_000i128);

    assert_eq!(token1.balance(&recipient), 5_000i128);
    assert_eq!(token4.balance(&recipient), 0i128);

    let swap1 = Swap {
        offer_asset: token1.address.clone(),
        ask_asset: token2.address.clone(),
    };
    let swap2 = Swap {
        offer_asset: token2.address.clone(),
        ask_asset: token3.address.clone(),
    };
    let swap3 = Swap {
        offer_asset: token3.address.clone(),
        ask_asset: token4.address.clone(),
    };

    let operations = vec![&env, swap1, swap2, swap3];

    multihop.swap(&recipient, &operations, &5_000i128);

    // 5. check if it goes according to plan
    assert_eq!(token1.balance(&recipient), 0i128);
    // because of the
    assert_eq!(
        token4.balance(&recipient),
        4_956i128,
        "token4 not as expected"
    );
}

// #[test]
// fn swap_three_different_pools_with_fees() {
//     let env = Env::default();
//
//     let admin = Address::random(&env);
//
//     env.mock_all_auths();
//     env.budget().reset_unlimited();
//
//     let mut token1 = deploy_and_mint_tokens(&env, &admin, 10_000_000i128);
//     let mut token2 = deploy_and_mint_tokens(&env, &admin, 10_000_000i128);
//     let mut token3 = deploy_and_mint_tokens(&env, &admin, 10_000_000i128);
//     let mut token4 = deploy_and_mint_tokens(&env, &admin, 10_000_000i128);
//
//     let mut tokens = [&mut token1, &mut token2, &mut token3, &mut token4];
//     tokens.sort_by(|a, b| a.address.cmp(&b.address));
//
//     // 1. deploy factory
//     let factory_client = deploy_and_initialize_factory(&env, admin.clone());
//
//     // 2. create liquidity pool from factory
//     let lp_wasm_hash = install_lp_contract(&env);
//
//     let first_token_init_info = TokenInitInfo {
//         token_wasm_hash: install_token_wasm(&env),
//         token_a: tokens[0].address.clone(),
//         token_b: tokens[1].address.clone(),
//     };
//     let first_stake_init_info = StakeInitInfo {
//         stake_wasm_hash: install_stake_wasm(&env),
//         min_bond: 10i128,
//         max_distributions: 10u32,
//         min_reward: 5i128,
//     };
//
//     let first_lp_init_info = LiquidityPoolInitInfo {
//         admin: admin.clone(),
//         fee_recipient: admin.clone(),
//         lp_wasm_hash: lp_wasm_hash.clone(),
//         max_allowed_slippage_bps: 5000,
//         max_allowed_spread_bps: 500,
//         share_token_decimals: 7,
//         swap_fee_bps: 1500,
//         token_init_info: first_token_init_info.clone(),
//         stake_init_info: first_stake_init_info,
//     };
//
//     let second_token_init_info = TokenInitInfo {
//         token_wasm_hash: install_token_wasm(&env),
//         token_a: tokens[1].address.clone(),
//         token_b: tokens[2].address.clone(),
//     };
//     let second_stake_init_info = StakeInitInfo {
//         stake_wasm_hash: install_stake_wasm(&env),
//         min_bond: 5i128,
//         max_distributions: 5u32,
//         min_reward: 2i128,
//     };
//
//     let second_lp_init_info = LiquidityPoolInitInfo {
//         admin: admin.clone(),
//         fee_recipient: admin.clone(),
//         lp_wasm_hash: lp_wasm_hash.clone(),
//         max_allowed_slippage_bps: 4_000,
//         max_allowed_spread_bps: 400,
//         share_token_decimals: 6,
//         swap_fee_bps: 1500,
//         token_init_info: second_token_init_info,
//         stake_init_info: second_stake_init_info,
//     };
//
//     let third_token_init_info = TokenInitInfo {
//         token_wasm_hash: install_token_wasm(&env),
//         token_a: tokens[2].address.clone(),
//         token_b: tokens[3].address.clone(),
//     };
//     let third_stake_init_info = StakeInitInfo {
//         stake_wasm_hash: install_stake_wasm(&env),
//         min_bond: 6i128,
//         max_distributions: 6u32,
//         min_reward: 3i128,
//     };
//
//     let third_lp_init_info = LiquidityPoolInitInfo {
//         admin: admin.clone(),
//         fee_recipient: admin.clone(),
//         lp_wasm_hash,
//         max_allowed_slippage_bps: 4_000,
//         max_allowed_spread_bps: 400,
//         share_token_decimals: 6,
//         swap_fee_bps: 1500,
//         token_init_info: third_token_init_info,
//         stake_init_info: third_stake_init_info,
//     };
//
//     let first_lp = factory_client.create_liquidity_pool(&first_lp_init_info);
//     let second_lp = factory_client.create_liquidity_pool(&second_lp_init_info);
//     let third_lp = factory_client.create_liquidity_pool(&third_lp_init_info);
//
//     let pools = [first_lp, second_lp, third_lp];
//
//     // 3. provide liquidity for each one of the liquidity pools
//     for pool in pools.iter() {
//         let lp_client = lp_contract::Client::new(&env, pool);
//         lp_client.provide_liquidity(
//             &admin.clone(),
//             &Some(1_000_000i128),
//             &None,
//             &Some(1_000_000i128),
//             &None,
//             &None::<i64>,
//         );
//     }
//
//     // 4. swap with multihop
//     let multihop = deploy_multihop_contract(&env, admin.clone(), &factory_client.address);
//     let recipient = Address::random(&env);
//     tokens[0].mint(&recipient, &50i128);
//     assert_eq!(tokens[0].balance(&recipient), 50i128);
//     assert_eq!(tokens[3].balance(&recipient), 0i128);
//
//     let swap1 = Swap {
//         offer_asset: tokens[0].address.clone(),
//         ask_asset: tokens[1].address.clone(),
//     };
//     let swap2 = Swap {
//         offer_asset: tokens[1].address.clone(),
//         ask_asset: tokens[2].address.clone(),
//     };
//     let swap3 = Swap {
//         offer_asset: tokens[2].address.clone(),
//         ask_asset: tokens[3].address.clone(),
//     };
//
//     let operations = vec![&env, swap1, swap2, swap3];
//
//     multihop.swap(&recipient, &operations, &50i128);
//
//     // 5. check if it goes according to plan
//     assert_eq!(tokens[0].balance(&recipient), 0i128);
//     assert_eq!(tokens[3].balance(&recipient), 247i128);
// }

#[test]
fn swap_panics_with_no_operations() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::random(&env);
    let factory = Address::random(&env);

    let recipient = Address::random(&env);

    let multihop = deploy_multihop_contract(&env, admin, &factory);

    let swap_vec = vec![&env];

    assert_eq!(
        multihop.try_swap(&recipient, &swap_vec, &50i128),
        Err(Ok(ContractError::OperationsEmpty))
    );
}

fn deploy_and_mint_tokens<'a>(
    env: &'a Env,
    admin: &'a Address,
    amount: i128,
) -> token_contract::Client<'a> {
    let token = deploy_token_contract(env, admin);
    token.mint(admin, &amount);
    token
}

fn deploy_and_initialize_factory(env: &Env, admin: Address) -> factory::Client {
    let factory_addr = deploy_factory_contract(env, admin.clone());
    let factory_client = factory::Client::new(env, &factory_addr);

    factory_client.initialize(&admin.clone());
    factory_client
}

#[allow(clippy::too_many_arguments)]
fn deploy_and_initialize_lp(
    env: &Env,
    factory: &factory::Client,
    admin: Address,
    mut token_a: Address,
    mut token_a_amount: i128,
    mut token_b: Address,
    mut token_b_amount: i128,
    fees: Option<i64>,
) {
    // 2. create liquidity pool from factory
    let lp_wasm_hash = install_lp_contract(env);

    if token_b < token_a {
        std::mem::swap(&mut token_a, &mut token_b);
        std::mem::swap(&mut token_a_amount, &mut token_b_amount);
    }
    let token_init_info = TokenInitInfo {
        token_wasm_hash: install_token_wasm(env),
        token_a: token_a.clone(),
        token_b: token_b.clone(),
    };
    let stake_init_info = StakeInitInfo {
        stake_wasm_hash: install_stake_wasm(env),
        min_bond: 10i128,
        max_distributions: 10u32,
        min_reward: 5i128,
    };

    let lp_init_info = LiquidityPoolInitInfo {
        admin: admin.clone(),
        fee_recipient: admin.clone(),
        lp_wasm_hash: lp_wasm_hash.clone(),
        max_allowed_slippage_bps: 5000,
        max_allowed_spread_bps: 500,
        share_token_decimals: 7,
        swap_fee_bps: fees.unwrap_or(0i64),
        token_init_info,
        stake_init_info,
    };

    let lp = factory.create_liquidity_pool(&lp_init_info);

    let lp_client = lp_contract::Client::new(env, &lp);
    lp_client.provide_liquidity(
        &admin.clone(),
        &Some(token_a_amount),
        &None,
        &Some(token_b_amount),
        &None,
        &None::<i64>,
    );
}
