// FIXM: Disable Referral struct
// use crate::lp_contract::Referral;
use crate::storage::Swap;
use crate::tests::setup::{
    deploy_and_initialize_factory, deploy_and_initialize_lp, deploy_and_mint_tokens,
    deploy_multihop_contract, deploy_token_contract,
};

use soroban_sdk::contracterror;
use soroban_sdk::{testutils::Address as _, vec, Address, Env};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    SpreadExceedsLimit = 1,
}

#[test]
fn swap_three_equal_pools_no_fees() {
    let env = Env::default();

    let admin = Address::generate(&env);

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
    let recipient = Address::generate(&env);
    token1.mint(&recipient, &50i128);
    assert_eq!(token1.balance(&recipient), 50i128);
    assert_eq!(token4.balance(&recipient), 0i128);

    let swap1 = Swap {
        offer_asset: token1.address.clone(),
        ask_asset: token2.address.clone(),
        max_belief_price: None::<i64>,
    };
    let swap2 = Swap {
        offer_asset: token2.address.clone(),
        ask_asset: token3.address.clone(),
        max_belief_price: None::<i64>,
    };
    let swap3 = Swap {
        offer_asset: token3.address.clone(),
        ask_asset: token4.address.clone(),
        max_belief_price: None::<i64>,
    };

    let operations = vec![&env, swap1, swap2, swap3];

    // FIXM: Disable Referral struct
    // multihop.swap(&recipient, &None, &operations, &None, &None, &50i128);
    multihop.swap(&recipient, &operations, &None, &50i128);

    // 5. check if it goes according to plan
    assert_eq!(token1.balance(&recipient), 0i128);
    assert_eq!(token4.balance(&recipient), 50i128);
}

// FIXM: Disable Referral struct
#[ignore]
#[test]
fn swap_three_equal_pools_no_fees_referral_fee() {
    let env = Env::default();

    let admin = Address::generate(&env);

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
    let recipient = Address::generate(&env);
    token1.mint(&recipient, &50i128);
    assert_eq!(token1.balance(&recipient), 50i128);
    assert_eq!(token4.balance(&recipient), 0i128);

    let swap1 = Swap {
        offer_asset: token1.address.clone(),
        ask_asset: token2.address.clone(),
        max_belief_price: None::<i64>,
    };
    let swap2 = Swap {
        offer_asset: token2.address.clone(),
        ask_asset: token3.address.clone(),
        max_belief_price: None::<i64>,
    };
    let swap3 = Swap {
        offer_asset: token3.address.clone(),
        ask_asset: token4.address.clone(),
        max_belief_price: None::<i64>,
    };

    let operations = vec![&env, swap1, swap2, swap3];
    let referral_addr = Address::generate(&env);
    // FIXM: Disable Referral struct
    // let referral = Referral {
    //     address: referral_addr.clone(),
    //     fee: 1_000,
    // };

    // multihop.swap(
    //     &recipient,
    //     &Some(referral),
    //     &operations,
    //     &None,
    //     &None,
    //     &50i128,
    // );
    multihop.swap(&recipient, &operations, &None, &50i128);

    // 5. check if it goes according to plan
    assert_eq!(token1.balance(&recipient), 0i128);
    assert_eq!(token4.balance(&recipient), 37i128);
    // referral fee from first swap should be 5 (10% out of 50)
    assert_eq!(token2.balance(&referral_addr), 5i128);
    // referral fee from 2nd swap should be 4 (10% out of 45) rounded down
    assert_eq!(token3.balance(&referral_addr), 4i128);
    // referral fee from the last swap should also be 4 (10% out of 41) rounded down
    assert_eq!(token4.balance(&referral_addr), 4i128);
}

#[test]
fn swap_single_pool_no_fees() {
    let env = Env::default();
    let admin = Address::generate(&env);

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
    let recipient = Address::generate(&env);
    token1.mint(&recipient, &5_000i128); // mints 50 token0 to recipient
    assert_eq!(token1.balance(&recipient), 5_000i128);
    assert_eq!(token2.balance(&recipient), 0i128);

    let swap1 = Swap {
        offer_asset: token1.address.clone(),
        ask_asset: token2.address.clone(),
        max_belief_price: None::<i64>,
    };

    let operations = vec![&env, swap1];

    // FIXM: Disable Referral struct
    // multihop.swap(&recipient, /*&None,*/ &operations, &None, &None, &50i128);
    multihop.swap(&recipient, &operations, &None, &1_000);

    // 5. check if it goes according to plan
    assert_eq!(token1.balance(&recipient), 4_000i128); // -1_000 token0
    assert_eq!(token2.balance(&recipient), 1_000i128); // +1_000 token1
}

#[test]
/// Asserting HostError, because of panic messages are not propagated and IIUC are normally compiled out
#[should_panic(expected = "HostError: Error(Contract, #1)")]
fn swap_should_fail_when_spread_exceeds_the_limit() {
    let env = Env::default();
    let admin = Address::generate(&env);

    env.mock_all_auths();
    env.budget().reset_unlimited();

    let token1 = deploy_and_mint_tokens(&env, &admin, 1_001_000i128);
    let token2 = deploy_and_mint_tokens(&env, &admin, 3_001_000i128);

    let factory_client = deploy_and_initialize_factory(&env, admin.clone());

    deploy_and_initialize_lp(
        &env,
        &factory_client,
        admin.clone(),
        token1.address.clone(),
        5_000,
        token2.address.clone(),
        2_000_000,
        None,
    );

    let multihop = deploy_multihop_contract(&env, admin.clone(), &factory_client.address);
    let recipient = Address::generate(&env);
    token1.mint(&recipient, &5_000i128); // mints 50 token0 to recipient

    let swap1 = Swap {
        offer_asset: token1.address.clone(),
        ask_asset: token2.address.clone(),
        max_belief_price: None::<i64>,
    };

    let operations = vec![&env, swap1];

    // FIXM: Disable Referral struct
    // multihop.swap(&recipient, &None, &operations, &None, &Some(50), &50);
    multihop.swap(&recipient, &operations, &Some(50), &50);
}

#[test]
fn swap_single_pool_with_fees() {
    let env = Env::default();
    let admin = Address::generate(&env);

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
    let recipient = Address::generate(&env);
    token1.mint(&recipient, &1000i128);
    assert_eq!(token1.balance(&recipient), 1000i128);
    assert_eq!(token2.balance(&recipient), 0i128);

    let swap1 = Swap {
        offer_asset: token1.address.clone(),
        ask_asset: token2.address.clone(),
        max_belief_price: None::<i64>,
    };

    let operations = vec![&env, swap1];

    // FIXM: Disable Referral struct
    // multihop.swap(&recipient, &None, &operations, &None, &None, &300i128);
    multihop.swap(&recipient, &operations, &None, &300i128);

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

    let admin = Address::generate(&env);

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
    let recipient = Address::generate(&env);
    token1.mint(&recipient, &5_000i128);

    assert_eq!(token1.balance(&recipient), 5_000i128);
    assert_eq!(token4.balance(&recipient), 0i128);

    let swap1 = Swap {
        offer_asset: token1.address.clone(),
        ask_asset: token2.address.clone(),
        max_belief_price: None::<i64>,
    };
    let swap2 = Swap {
        offer_asset: token2.address.clone(),
        ask_asset: token3.address.clone(),
        max_belief_price: None::<i64>,
    };
    let swap3 = Swap {
        offer_asset: token3.address.clone(),
        ask_asset: token4.address.clone(),
        max_belief_price: None::<i64>,
    };

    let operations = vec![&env, swap1, swap2, swap3];

    // FIXM: Disable Referral struct
    // multihop.swap(&recipient, &None, &operations, &None, &None, &5_000i128);
    multihop.swap(&recipient, &operations, &None, &5_000i128);

    // 5. check if it goes according to plan
    assert_eq!(token1.balance(&recipient), 0i128);
    assert_eq!(
        token4.balance(&recipient),
        4_956i128,
        "token4 not as expected"
    );
}

#[test]
fn swap_three_different_pools_with_fees() {
    let env = Env::default();

    let admin = Address::generate(&env);

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
        Some(1_000),
    );
    deploy_and_initialize_lp(
        &env,
        &factory_client,
        admin.clone(),
        token2.address.clone(),
        2_000_000,
        token3.address.clone(),
        2_000_000,
        Some(1_000),
    );
    deploy_and_initialize_lp(
        &env,
        &factory_client,
        admin.clone(),
        token3.address.clone(),
        3_000_000,
        token4.address.clone(),
        3_000_000,
        Some(1_000),
    );

    // 4. swap with multihop
    let multihop = deploy_multihop_contract(&env, admin.clone(), &factory_client.address);
    let recipient = Address::generate(&env);
    token1.mint(&recipient, &10_000i128);
    assert_eq!(token1.balance(&recipient), 10_000i128);

    let swap1 = Swap {
        offer_asset: token1.address.clone(),
        ask_asset: token2.address.clone(),
        max_belief_price: None::<i64>,
    };
    let swap2 = Swap {
        offer_asset: token2.address.clone(),
        ask_asset: token3.address.clone(),
        max_belief_price: None::<i64>,
    };
    let swap3 = Swap {
        offer_asset: token3.address.clone(),
        ask_asset: token4.address.clone(),
        max_belief_price: None::<i64>,
    };

    let operations = vec![&env, swap1, swap2, swap3];

    // FIXM: Disable Referral struct
    // multihop.swap(&recipient, &None, &operations, &None, &None, &10_000i128);
    multihop.swap(&recipient, &operations, &None, &10_000i128);

    // we start swapping 10_000 tokens

    // token1 => token2
    // (10_000 * 1_000_000) / (10_000 + 1_000_000)
    // 10_000_000_000 / 1_010_000
    // 9900.99009901
    // 9901 - 10% =  8911

    // token2 => token3
    // (8911 * 2_000_000) / (8911 + 2_000_000)
    // 17_822_000_000 / 2_008_911
    // 8871.47315137
    // 8872 - 10% = 7985

    // token3 => token4
    // (7985 * 3_000_000) / (7985 + 3_000_000)
    // 23_955_000_000 / 3_007_985
    // 7963.80301099
    // 7964 - 10% = 7168
    assert_eq!(token1.balance(&recipient), 0i128);
    assert_eq!(token2.balance(&recipient), 0i128);
    assert_eq!(token3.balance(&recipient), 0i128);
    assert_eq!(token4.balance(&recipient), 7_168i128);
}

#[test]
#[should_panic(expected = "Multihop: Swap: operations is empty!")]
fn swap_panics_with_no_operations() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let factory = Address::generate(&env);
    let recipient = Address::generate(&env);

    let token = deploy_token_contract(&env, &admin);
    token.mint(&recipient, &50i128);

    let multihop = deploy_multihop_contract(&env, admin, &factory);

    let swap_vec = vec![&env];

    // FIXM: Disable Referral struct
    // multihop.swap(&recipient, &None, &swap_vec, &None, &None, &50i128);
    multihop.swap(&recipient, &swap_vec, &None, &50i128);
}

#[test]
fn test_v_phx_vul_013_add_belief_price_for_every_swap() {
    let env = Env::default();

    let admin = Address::generate(&env);

    env.mock_all_auths();
    env.budget().reset_unlimited();

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
    let recipient = Address::generate(&env);
    token1.mint(&recipient, &5_000i128);

    let swap1 = Swap {
        offer_asset: token1.address.clone(),
        ask_asset: token2.address.clone(),
        max_belief_price: Some(1_050),
    };
    let swap2 = Swap {
        offer_asset: token2.address.clone(),
        ask_asset: token3.address.clone(),
        max_belief_price: Some(2_100),
    };
    let swap3 = Swap {
        offer_asset: token3.address.clone(),
        ask_asset: token4.address.clone(),
        max_belief_price: Some(3_150),
    };

    let operations = vec![&env, swap1, swap2, swap3];

    multihop.swap(&recipient, &operations, &None, &5_000i128);

    assert_eq!(
        token1.balance(&recipient),
        0i128,
        "token1 balance incorrect"
    );
    assert_eq!(
        token4.balance(&recipient),
        4_956i128,
        "token4 balance incorrect"
    );
}
