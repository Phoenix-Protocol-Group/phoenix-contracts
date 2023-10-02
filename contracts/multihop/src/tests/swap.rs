use crate::error::ContractError;
use crate::storage::Swap;
use crate::tests::setup::factory::{LiquidityPoolInitInfo, StakeInitInfo, TokenInitInfo};
use crate::tests::setup::{
    deploy_factory_contract, deploy_multihop_contract, deploy_token_contract, factory,
    install_lp_contract, install_stake_wasm, install_token_wasm,
};
use soroban_sdk::arbitrary::std;
use soroban_sdk::arbitrary::std::dbg;
use soroban_sdk::{testutils::Address as _, vec, Address, Env};

#[test]
fn test_swap() {
    let env = Env::default();
    let admin = Address::random(&env);
    let user = Address::random(&env);

    let mut token1 = deploy_token_contract(&env, &admin);
    let mut token2 = deploy_token_contract(&env, &admin);
    let mut token3 = deploy_token_contract(&env, &admin);
    let mut token4 = deploy_token_contract(&env, &admin);
    let mut token5 = deploy_token_contract(&env, &admin);
    let mut token6 = deploy_token_contract(&env, &admin);

    env.mock_all_auths();
    env.budget().reset_unlimited();

    if token2.address < token1.address {
        std::mem::swap(&mut token1, &mut token2);
    }

    if token4.address < token3.address {
        std::mem::swap(&mut token3, &mut token4);
    }

    if token6.address < token5.address {
        std::mem::swap(&mut token5, &mut token6);
    }

    token1.mint(&user, &10_000i128);
    token2.mint(&user, &10_000i128);
    token3.mint(&user, &10_000i128);
    token4.mint(&user, &10_000i128);
    token5.mint(&user, &10_000i128);
    token6.mint(&user, &10_000i128);

    // 1. deploy factory
    let factory_addr = deploy_factory_contract(&env, admin.clone());
    let factory_client = factory::Client::new(&env, &factory_addr);

    factory_client.initialize(&admin.clone());

    // 2. create liquidity pool from factory
    let lp_wasm_hash = install_lp_contract(&env);

    let first_token_init_info = TokenInitInfo {
        token_wasm_hash: install_token_wasm(&env),
        token_a: token1.address.clone(),
        token_b: token2.address.clone(),
    };
    let first_stake_init_info = StakeInitInfo {
        stake_wasm_hash: install_stake_wasm(&env),
        min_bond: 10i128,
        max_distributions: 10u32,
        min_reward: 5i128,
    };

    let first_lp_init_info = LiquidityPoolInitInfo {
        admin: admin.clone(),
        fee_recipient: user.clone(),
        lp_wasm_hash: lp_wasm_hash.clone(),
        max_allowed_slippage_bps: 5_000,
        max_allowed_spread_bps: 500,
        share_token_decimals: 7,
        swap_fee_bps: 0,
        token_init_info: first_token_init_info.clone(),
        stake_init_info: first_stake_init_info,
    };

    let second_token_init_info = TokenInitInfo {
        token_wasm_hash: install_token_wasm(&env),
        token_a: token3.address.clone(),
        token_b: token4.address.clone(),
    };
    let second_stake_init_info = StakeInitInfo {
        stake_wasm_hash: install_stake_wasm(&env),
        min_bond: 5i128,
        max_distributions: 5u32,
        min_reward: 2i128,
    };

    let second_lp_init_info = LiquidityPoolInitInfo {
        admin: admin.clone(),
        fee_recipient: user.clone(),
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
        token_a: token5.address.clone(),
        token_b: token6.address.clone(),
    };
    let third_stake_init_info = StakeInitInfo {
        stake_wasm_hash: install_stake_wasm(&env),
        min_bond: 6i128,
        max_distributions: 6u32,
        min_reward: 3i128,
    };

    let third_lp_init_info = LiquidityPoolInitInfo {
        admin: admin.clone(),
        fee_recipient: user.clone(),
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

    dbg!(&factory_client.query_all_pools_details());

    // 3. use the swap method of multihop
    let multihop = deploy_multihop_contract(&env, admin, factory_client.address);
    //
    let recipient = Address::random(&env);

    let swap1 = Swap {
        ask_asset: token1.address,
        offer_asset: token2.address,
    };
    let swap2 = Swap {
        ask_asset: token3.address,
        offer_asset: token4.address,
    };
    let swap3 = Swap {
        ask_asset: token5.address,
        offer_asset: token6.address,
    };

    dbg!(&swap1);
    dbg!(&swap2);
    dbg!(&swap3);

    let swap_vec = vec![&env, swap1, swap2, swap3];
    // WHY WOULD &swap_vec BE MARKED BY THE COMPILER LIKE THAT...
    multihop.swap(&recipient, &swap_vec, &5i128);

    // 4. check if it goes according to plan
}

#[test]
fn test_swap_should_return_err() {
    let env = Env::default();
    let admin = Address::random(&env);
    let factory = Address::random(&env);

    let multihop = deploy_multihop_contract(&env, admin, factory);

    let recipient = Address::random(&env);

    let swap_vec = vec![&env];

    assert_eq!(
        multihop.try_swap(&recipient, &swap_vec, &5i128),
        Err(Ok(ContractError::OperationsEmpty))
    );
}
