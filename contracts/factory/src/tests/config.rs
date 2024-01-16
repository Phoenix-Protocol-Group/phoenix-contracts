use super::setup::{
    deploy_factory_contract, install_lp_contract, install_stake_wasm, install_token_wasm,
    lp_contract,
};
use phoenix::utils::{LiquidityPoolInitInfo, StakeInitInfo, TokenInitInfo};

use soroban_sdk::{testutils::arbitrary::std, Address, Env, String};

#[test]
fn factory_successfully_inits_itself() {
    let env = Env::default();
    let admin = Address::from_string(&String::from_str(
        &env,
        "CBT4WEAHQ72AYRD7WZFNYE6HGZEIX25754NG37LBLXTTRMWKQNKIUR6O",
    ));

    let factory = deploy_factory_contract(&env, Some(admin.clone()));

    assert_eq!(factory.get_admin(), admin);
}

#[test]
fn factory_successfully_inits_multihop() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::from_string(&String::from_str(
        &env,
        "CBT4WEAHQ72AYRD7WZFNYE6HGZEIX25754NG37LBLXTTRMWKQNKIUR6O",
    ));

    let factory = deploy_factory_contract(&env, Some(admin.clone()));

    let multihop_address = factory.get_config().multihop_address;

    let func = Symbol::new(&env, "get_admin");
    let admin_in_multihop = env.invoke_contract(&multihop_address, &func, Vec::new(&env));

    assert_eq!(admin, admin_in_multihop);
}

#[test]
fn factory_successfully_inits_lp() {
    let env = Env::default();
    let admin = Address::from_string(&String::from_str(
        &env,
        "CBT4WEAHQ72AYRD7WZFNYE6HGZEIX25754NG37LBLXTTRMWKQNKIUR6O",
    ));
    let mut token1_admin = Address::from_string(&String::from_str(
        &env,
        "CDJ4KQHEFNC5GBCRBKGIDGNLVVXORIDHXU5EYFQIXMDPJ3ACJVPXUOFY",
    ));
    let mut token2_admin = Address::from_string(&String::from_str(
        &env,
        "CC6TPVKWO2ZHGIMAIBPKE3RA74TGPCD6637FAT5NS3UCFDQ7OHMIIG6I",
    ));
    let user = Address::from_string(&String::from_str(
        &env,
        "CDUK25UHLE7LUDQZ4UTTNWMXABJHW76Q74SKOK6BMWGKDHIJ6MIBOK6N",
    ));

    let mut token1 = Address::from_string(&String::from_str(
        &env,
        "GCHM6Y4BYTUJZQ4KABLTC73IWDZNFBZ2NML5W4AQG7XAJEOMLF5I774H",
    ));
    let mut token2 = Address::from_string(&String::from_str(
        &env,
        "CBGJMPOZ573XUTIRRFWGWTGSIAOGKJRVMIKBTFYEWTEIU7AEDWKDYMUX",
    ));

    env.mock_all_auths();
    env.budget().reset_unlimited();

    if token2 < token1 {
        std::mem::swap(&mut token1, &mut token2);
        std::mem::swap(&mut token1_admin, &mut token2_admin);
    }

    let factory = deploy_factory_contract(&env, Some(admin.clone()));
    assert_eq!(factory.get_admin(), admin);

    let token_init_info = TokenInitInfo {
        token_wasm_hash: install_token_wasm(&env),
        token_a: token1,
        token_b: token2,
    };
    let stake_init_info = StakeInitInfo {
        stake_wasm_hash: install_stake_wasm(&env),
        min_bond: 10i128,
        max_distributions: 10u32,
        min_reward: 5i128,
    };

    let lp_wasm_hash = install_lp_contract(&env);

    let lp_init_info = LiquidityPoolInitInfo {
        admin,
        fee_recipient: user.clone(),
        lp_wasm_hash,
        max_allowed_slippage_bps: 5_000,
        max_allowed_spread_bps: 500,
        share_token_decimals: 7,
        swap_fee_bps: 0,
        max_referral_bps: 5_000,
        token_init_info: token_init_info.clone(),
        stake_init_info,
    };

    factory.create_liquidity_pool(
        &lp_init_info,
        &Address::from_contract_id(&BytesN::from_array(&env, &[0u8; 0x20])),
    );
    let lp_contract_addr = factory.query_pools().get(0).unwrap();

    let first_lp_contract = lp_contract::Client::new(&env, &lp_contract_addr);
    let share_token_address = first_lp_contract.query_share_token_address();
    let stake_token_address = first_lp_contract.query_stake_contract_address();

    assert_eq!(
        first_lp_contract.query_config(),
        lp_contract::Config {
            fee_recipient: user,
            max_allowed_slippage_bps: 5_000,
            max_allowed_spread_bps: 500,
            max_referral_bps: 5_000,
            pool_type: lp_contract::PairType::Xyk,
            share_token: share_token_address,
            stake_contract: stake_token_address,
            token_a: token_init_info.token_a,
            token_b: token_init_info.token_b,
            total_fee_bps: 0,
        }
    );
}

#[test]
#[should_panic]
fn factory_fails_to_init_lp_when_authorized_address_not_present() {
    let env = Env::default();
    let admin = Address::random(&env);
    let mut token1_admin = Address::random(&env);
    let mut token2_admin = Address::random(&env);
    let user = Address::random(&env);

    let mut token1 = Address::random(&env);
    let mut token2 = Address::random(&env);

    env.mock_all_auths();
    env.budget().reset_unlimited();

    if token2 < token1 {
        std::mem::swap(&mut token1, &mut token2);
        std::mem::swap(&mut token1_admin, &mut token2_admin);
    }

    let factory = deploy_factory_contract(&env, Some(admin.clone()));
    assert_eq!(factory.get_admin(), admin);

    let token_init_info = TokenInitInfo {
        token_wasm_hash: install_token_wasm(&env),
        token_a: token1,
        token_b: token2,
    };
    let stake_init_info = StakeInitInfo {
        stake_wasm_hash: install_stake_wasm(&env),
        min_bond: 10i128,
        max_distributions: 10u32,
        min_reward: 5i128,
    };

    let lp_wasm_hash = install_lp_contract(&env);

    let lp_init_info = LiquidityPoolInitInfo {
        admin,
        fee_recipient: user.clone(),
        lp_wasm_hash,
        max_allowed_slippage_bps: 5_000,
        max_allowed_spread_bps: 500,
        share_token_decimals: 7,
        swap_fee_bps: 0,
        max_referral_bps: 5_000,
        token_init_info: token_init_info.clone(),
        stake_init_info,
    };

    factory.create_liquidity_pool(&lp_init_info, &Address::random(&env));
}
