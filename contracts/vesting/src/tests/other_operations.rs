use curve::{Curve, SaturatingLinear};
use soroban_sdk::{testutils::Address as _, vec, Address, Env, String};

use crate::storage::{MinterInfo, VestingBalance, VestingTokenInfo};

use super::setup::{deploy_token_contract, instantiate_vesting_client};

#[test]
fn burn_should_work_correctly() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);
    let token = deploy_token_contract(&env, &admin);

    token.mint(&vester1, &1_000);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: token.address.clone(),
        total_supply: 0,
    };
    let vesting_balances = vec![
        &env,
        VestingBalance {
            address: vester1.clone(),
            balance: 1_000,
            curve: Curve::SaturatingLinear(SaturatingLinear {
                min_x: 15,
                min_y: 120,
                max_x: 60,
                max_y: 0,
            }),
        },
    ];

    let vesting_client = instantiate_vesting_client(&env);
    vesting_client.initialize(
        &admin,
        &vesting_token,
        &vesting_balances,
        &None,
        &None,
        &10u32,
    );

    assert_eq!(vesting_client.query_vesting_total_supply(), 1_000);

    vesting_client.burn(&vester1, &500);

    assert_eq!(vesting_client.query_vesting_total_supply(), 500);
    assert_eq!(token.balance(&vester1), 500);
}

#[test]
#[should_panic(expected = "Vesting: Burn: Invalid burn amount")]
fn burn_should_panic_when_invalid_amount() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);
    let token = deploy_token_contract(&env, &admin);

    token.mint(&vester1, &1_000);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: token.address.clone(),
        total_supply: 0,
    };
    let vesting_balances = vec![
        &env,
        VestingBalance {
            address: vester1.clone(),
            balance: 1_000,
            curve: Curve::SaturatingLinear(SaturatingLinear {
                min_x: 15,
                min_y: 120,
                max_x: 60,
                max_y: 0,
            }),
        },
    ];

    let vesting_client = instantiate_vesting_client(&env);
    vesting_client.initialize(
        &admin,
        &vesting_token,
        &vesting_balances,
        &None,
        &None,
        &10u32,
    );

    vesting_client.burn(&vester1, &0);
}

#[test]
#[should_panic(expected = "Vesting: Burn: Critical error - total supply cannot be negative")]
fn burn_should_panic_when_total_supply_becomes_negative() {
    const TOO_MUCH: i128 = 1_000;
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);
    let token = deploy_token_contract(&env, &admin);

    token.mint(&vester1, &1_000);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: token.address.clone(),
        total_supply: 0,
    };
    let vesting_balances = vec![
        &env,
        VestingBalance {
            address: vester1.clone(),
            balance: 500,
            curve: Curve::SaturatingLinear(SaturatingLinear {
                min_x: 15,
                min_y: 120,
                max_x: 60,
                max_y: 0,
            }),
        },
    ];

    let vesting_client = instantiate_vesting_client(&env);
    vesting_client.initialize(
        &admin,
        &vesting_token,
        &vesting_balances,
        &None,
        &None,
        &10u32,
    );

    vesting_client.burn(&vester1, &TOO_MUCH);
}

#[test]
fn mint_should_work_correctly() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);
    let rcpt = Address::generate(&env);

    let token = deploy_token_contract(&env, &admin);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: token.address.clone(),
        total_supply: 0,
    };
    let vesting_balances = vec![
        &env,
        VestingBalance {
            address: vester1.clone(),
            balance: 200,
            curve: Curve::SaturatingLinear(SaturatingLinear {
                min_x: 15,
                min_y: 120,
                max_x: 60,
                max_y: 0,
            }),
        },
    ];

    let minter_info = MinterInfo {
        address: vester1.clone(),
        capacity: Curve::Constant(500),
    };

    let vesting_client = instantiate_vesting_client(&env);
    vesting_client.initialize(
        &admin,
        &vesting_token,
        &vesting_balances,
        &Some(minter_info),
        &None,
        &10u32,
    );

    assert_eq!(token.balance(&rcpt), 0);
    assert_eq!(vesting_client.query_vesting_total_supply(), 200);

    vesting_client.mint(&vester1, &rcpt, &100);

    assert_eq!(token.balance(&rcpt), 100);
    assert_eq!(vesting_client.query_vesting_total_supply(), 300);
}

#[test]
#[should_panic(expected = "Vesting: Mint: Invalid mint amount")]
fn mint_should_panic_when_invalid_amount() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);
    let rcpt = Address::generate(&env);

    let token = deploy_token_contract(&env, &admin);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: token.address.clone(),
        total_supply: 0,
    };
    let vesting_balances = vec![
        &env,
        VestingBalance {
            address: vester1.clone(),
            balance: 200,
            curve: Curve::SaturatingLinear(SaturatingLinear {
                min_x: 15,
                min_y: 120,
                max_x: 60,
                max_y: 0,
            }),
        },
    ];

    let minter_info = MinterInfo {
        address: vester1.clone(),
        capacity: Curve::Constant(500),
    };

    let vesting_client = instantiate_vesting_client(&env);
    vesting_client.initialize(
        &admin,
        &vesting_token,
        &vesting_balances,
        &Some(minter_info),
        &None,
        &10u32,
    );

    vesting_client.mint(&vester1, &rcpt, &0);
}

#[test]
#[should_panic(expected = "Vesting: Mint: Not authorized to mint")]
fn mint_should_panic_when_not_authorized_to_mint() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);
    let rcpt = Address::generate(&env);

    let token = deploy_token_contract(&env, &admin);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: token.address.clone(),
        total_supply: 0,
    };
    let vesting_balances = vec![
        &env,
        VestingBalance {
            address: vester1.clone(),
            balance: 200,
            curve: Curve::SaturatingLinear(SaturatingLinear {
                min_x: 15,
                min_y: 120,
                max_x: 60,
                max_y: 0,
            }),
        },
    ];

    let minter_info = MinterInfo {
        address: Address::generate(&env),
        capacity: Curve::Constant(500),
    };

    let vesting_client = instantiate_vesting_client(&env);
    vesting_client.initialize(
        &admin,
        &vesting_token,
        &vesting_balances,
        &Some(minter_info),
        &None,
        &10u32,
    );

    vesting_client.mint(&vester1, &rcpt, &100);
}

#[test]
#[should_panic(expected = "Vesting: Mint: Critical error - total supply overflow")]
fn mint_should_panic_when_supply_overflow() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);
    let rcpt = Address::generate(&env);

    let too_generous = &i128::MAX;

    let token = deploy_token_contract(&env, &admin);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: token.address.clone(),
        total_supply: 0,
    };
    let vesting_balances = vec![
        &env,
        VestingBalance {
            address: vester1.clone(),
            balance: 200,
            curve: Curve::SaturatingLinear(SaturatingLinear {
                min_x: 15,
                min_y: 120,
                max_x: 60,
                max_y: 0,
            }),
        },
    ];

    let minter_info = MinterInfo {
        address: vester1.clone(),
        capacity: Curve::Constant(500),
    };

    let vesting_client = instantiate_vesting_client(&env);
    vesting_client.initialize(
        &admin,
        &vesting_token,
        &vesting_balances,
        &Some(minter_info),
        &None,
        &10u32,
    );

    vesting_client.mint(&vester1, &rcpt, too_generous);
}

#[test]
#[should_panic(expected = "Vesting: Mint: total supply over the capacity")]
fn mint_should_panic_when_mint_over_the_cap() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);
    let rcpt = Address::generate(&env);

    let token = deploy_token_contract(&env, &admin);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: token.address.clone(),
        total_supply: 0,
    };
    let vesting_balances = vec![
        &env,
        VestingBalance {
            address: vester1.clone(),
            balance: 200,
            curve: Curve::SaturatingLinear(SaturatingLinear {
                min_x: 15,
                min_y: 120,
                max_x: 60,
                max_y: 0,
            }),
        },
    ];

    let minter_info = MinterInfo {
        address: vester1.clone(),
        capacity: Curve::Constant(500),
    };

    let vesting_client = instantiate_vesting_client(&env);
    vesting_client.initialize(
        &admin,
        &vesting_token,
        &vesting_balances,
        &Some(minter_info),
        &None,
        &10u32,
    );

    vesting_client.mint(&vester1, &rcpt, &500);
}

#[test]
fn update_minter_works_correctly() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);
    let new_minter = Address::generate(&env);

    let token = deploy_token_contract(&env, &admin);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: token.address.clone(),
        total_supply: 0,
    };
    let vesting_balances = vec![
        &env,
        VestingBalance {
            address: vester1.clone(),
            balance: 200,
            curve: Curve::SaturatingLinear(SaturatingLinear {
                min_x: 15,
                min_y: 120,
                max_x: 60,
                max_y: 0,
            }),
        },
    ];

    let minter_info = MinterInfo {
        address: vester1.clone(),
        capacity: Curve::Constant(500),
    };

    let vesting_client = instantiate_vesting_client(&env);
    vesting_client.initialize(
        &admin,
        &vesting_token,
        &vesting_balances,
        &Some(minter_info.clone()),
        &None,
        &10u32,
    );

    assert_eq!(vesting_client.query_minter(), minter_info);

    let new_minter_info = MinterInfo {
        address: new_minter.clone(),
        capacity: Curve::Constant(1_000),
    };

    vesting_client.update_minter(&vester1, &new_minter_info.address);

    assert_eq!(
        vesting_client.query_minter().address,
        new_minter_info.address
    );
}

#[test]
#[should_panic(expected = "Vesting: Update minter: Not authorized to update minter")]
fn update_minter_fails_when_not_authorized() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);
    let new_minter = Address::generate(&env);

    let token = deploy_token_contract(&env, &admin);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: token.address.clone(),
        total_supply: 0,
    };
    let vesting_balances = vec![
        &env,
        VestingBalance {
            address: vester1.clone(),
            balance: 200,
            curve: Curve::SaturatingLinear(SaturatingLinear {
                min_x: 15,
                min_y: 120,
                max_x: 60,
                max_y: 0,
            }),
        },
    ];

    let minter_info = MinterInfo {
        address: Address::generate(&env),
        capacity: Curve::Constant(500),
    };

    let vesting_client = instantiate_vesting_client(&env);
    vesting_client.initialize(
        &admin,
        &vesting_token,
        &vesting_balances,
        &Some(minter_info.clone()),
        &None,
        &10u32,
    );

    let new_minter_info = MinterInfo {
        address: new_minter.clone(),
        capacity: Curve::Constant(1_000),
    };

    vesting_client.update_minter(&vester1, &new_minter_info.address);
}

#[test]
fn test_add_to_whitelist_should_work() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);
    let whitelisted1 = Address::generate(&env);
    let whitelisted2 = Address::generate(&env);

    let token = deploy_token_contract(&env, &admin);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: token.address.clone(),
        total_supply: 0,
    };

    let vesting_balances = vec![
        &env,
        VestingBalance {
            address: vester1.clone(),
            balance: 200,
            curve: Curve::SaturatingLinear(SaturatingLinear {
                min_x: 15,
                min_y: 120,
                max_x: 60,
                max_y: 0,
            }),
        },
    ];

    let vesting_client = instantiate_vesting_client(&env);
    vesting_client.initialize(
        &admin,
        &vesting_token,
        &vesting_balances,
        &None,
        &None,
        &10u32,
    );

    // no one is whitelisted initially, so we end up with just the admin inside the list
    assert_eq!(
        vesting_client.query_vesting_whitelist(),
        vec![&env, admin.clone()]
    );

    vesting_client.add_to_whitelist(
        &admin,
        &vec![
            &env,
            vester1.clone(),
            whitelisted1.clone(),
            whitelisted2.clone(),
        ],
    );

    assert_eq!(
        vesting_client.query_vesting_whitelist(),
        vec![&env, admin, vester1, whitelisted1, whitelisted2]
    );
}

#[test]
#[should_panic(expected = "Vesting: Add to whitelist: Not authorized to add to whitelist")]
fn test_add_to_whitelist_should_fail_when_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);

    let token = deploy_token_contract(&env, &admin);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: token.address.clone(),
        total_supply: 0,
    };

    let vesting_balances = vec![
        &env,
        VestingBalance {
            address: vester1.clone(),
            balance: 200,
            curve: Curve::SaturatingLinear(SaturatingLinear {
                min_x: 15,
                min_y: 120,
                max_x: 60,
                max_y: 0,
            }),
        },
    ];

    let vesting_client = instantiate_vesting_client(&env);
    vesting_client.initialize(
        &admin,
        &vesting_token,
        &vesting_balances,
        &None,
        &None,
        &10u32,
    );

    // no one is whitelisted initially, so we end up with just the admin inside the list
    assert_eq!(
        vesting_client.query_vesting_whitelist(),
        vec![&env, admin.clone()]
    );

    vesting_client.add_to_whitelist(&Address::generate(&env), &vec![&env, vester1]);
}

#[test]
fn test_remove_from_whitelist_should_work() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);
    let transit_account = Address::generate(&env);

    let token = deploy_token_contract(&env, &admin);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: token.address.clone(),
        total_supply: 0,
    };

    let vesting_balances = vec![
        &env,
        VestingBalance {
            address: vester1.clone(),
            balance: 200,
            curve: Curve::SaturatingLinear(SaturatingLinear {
                min_x: 15,
                min_y: 120,
                max_x: 60,
                max_y: 0,
            }),
        },
    ];

    let vesting_client = instantiate_vesting_client(&env);
    vesting_client.initialize(
        &admin,
        &vesting_token,
        &vesting_balances,
        &None,
        &None,
        &10u32,
    );

    // no one is whitelisted initially, so we end up with just the admin inside the list
    assert_eq!(
        vesting_client.query_vesting_whitelist(),
        vec![&env, admin.clone()]
    );

    vesting_client.add_to_whitelist(
        &admin,
        &vec![&env, vester1.clone(), transit_account.clone()],
    );

    assert_eq!(
        vesting_client.query_vesting_whitelist(),
        vec![
            &env,
            admin.clone(),
            vester1.clone(),
            transit_account.clone()
        ]
    );

    vesting_client.remove_from_whitelist(&admin, &transit_account);

    assert_eq!(
        vesting_client.query_vesting_whitelist(),
        vec![&env, admin, vester1]
    );
}

#[test]
#[should_panic(
    expected = "Vesting: Remove from whitelist: Not authorized to remove from whitelist"
)]
fn test_should_remove_from_whitelist_should_fail_when_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);

    let token = deploy_token_contract(&env, &admin);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: token.address.clone(),
        total_supply: 0,
    };

    let vesting_balances = vec![
        &env,
        VestingBalance {
            address: vester1.clone(),
            balance: 200,
            curve: Curve::SaturatingLinear(SaturatingLinear {
                min_x: 15,
                min_y: 120,
                max_x: 60,
                max_y: 0,
            }),
        },
    ];

    let vesting_client = instantiate_vesting_client(&env);
    vesting_client.initialize(
        &admin,
        &vesting_token,
        &vesting_balances,
        &None,
        &None,
        &10u32,
    );

    // no one is whitelisted initially, so we end up with just the admin inside the list
    assert_eq!(
        vesting_client.query_vesting_whitelist(),
        vec![&env, admin.clone()]
    );

    vesting_client.add_to_whitelist(&admin, &vec![&env, vester1.clone()]);

    assert_eq!(
        vesting_client.query_vesting_whitelist(),
        vec![&env, admin.clone(), vester1.clone()]
    );

    vesting_client.remove_from_whitelist(&Address::generate(&env), &admin);
}

#[test]
#[should_panic(executed = "Vesting: Add to whitelist: No addresses to add")]
fn test_should_panic_when_no_addresses_to_add() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);

    let token = deploy_token_contract(&env, &admin);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: token.address.clone(),
        total_supply: 0,
    };

    let vesting_balances = vec![
        &env,
        VestingBalance {
            address: vester1.clone(),
            balance: 200,
            curve: Curve::SaturatingLinear(SaturatingLinear {
                min_x: 15,
                min_y: 120,
                max_x: 60,
                max_y: 0,
            }),
        },
    ];

    let vesting_client = instantiate_vesting_client(&env);
    vesting_client.initialize(
        &admin,
        &vesting_token,
        &vesting_balances,
        &None,
        &None,
        &10u32,
    );

    vesting_client.add_to_whitelist(&admin, &vec![&env]);
}

#[test]
fn test_should_update_minter_capacity_when_replacing_old_capacity() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);

    let token = deploy_token_contract(&env, &admin);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: token.address.clone(),
        total_supply: 0,
    };

    let vesting_balances = vec![
        &env,
        VestingBalance {
            address: vester1.clone(),
            balance: 200,
            curve: Curve::SaturatingLinear(SaturatingLinear {
                min_x: 15,
                min_y: 120,
                max_x: 60,
                max_y: 0,
            }),
        },
    ];

    let minter_info = MinterInfo {
        address: vester1.clone(),
        capacity: Curve::Constant(500),
    };

    let vesting_client = instantiate_vesting_client(&env);
    vesting_client.initialize(
        &admin,
        &vesting_token,
        &vesting_balances,
        &Some(minter_info.clone()),
        &None,
        &10u32,
    );

    let new_capacity = Curve::Constant(1_000);
    vesting_client.update_minter_capacity(&admin, &new_capacity, &true);

    assert_eq!(vesting_client.query_minter().capacity, new_capacity);
}

#[test]
fn test_should_update_minter_capacity_when_combining_old_capacity() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);

    let token = deploy_token_contract(&env, &admin);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: token.address.clone(),
        total_supply: 0,
    };

    let vesting_balances = vec![
        &env,
        VestingBalance {
            address: vester1.clone(),
            balance: 200,
            curve: Curve::SaturatingLinear(SaturatingLinear {
                min_x: 15,
                min_y: 120,
                max_x: 60,
                max_y: 0,
            }),
        },
    ];

    let minter_info = MinterInfo {
        address: vester1.clone(),
        capacity: Curve::Constant(500),
    };

    let vesting_client = instantiate_vesting_client(&env);
    vesting_client.initialize(
        &admin,
        &vesting_token,
        &vesting_balances,
        &Some(minter_info.clone()),
        &None,
        &10u32,
    );

    let new_capacity = Curve::Constant(1_000);
    vesting_client.update_minter_capacity(&admin, &new_capacity, &false);

    assert_eq!(
        vesting_client.query_minter().capacity,
        Curve::Constant(1_500)
    );
}

#[test]
#[should_panic(
    expected = "Vesting: Update minter capacity: Not authorized to update minter capacity"
)]
fn test_should_panic_when_updating_minter_capacity_without_auth() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);

    let token = deploy_token_contract(&env, &admin);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: token.address.clone(),
        total_supply: 0,
    };

    let vesting_balances = vec![
        &env,
        VestingBalance {
            address: vester1.clone(),
            balance: 200,
            curve: Curve::SaturatingLinear(SaturatingLinear {
                min_x: 15,
                min_y: 120,
                max_x: 60,
                max_y: 0,
            }),
        },
    ];

    let minter_info = MinterInfo {
        address: vester1.clone(),
        capacity: Curve::Constant(500),
    };

    let vesting_client = instantiate_vesting_client(&env);
    vesting_client.initialize(
        &admin,
        &vesting_token,
        &vesting_balances,
        &Some(minter_info.clone()),
        &None,
        &10u32,
    );

    vesting_client.update_minter_capacity(&Address::generate(&env), &Curve::Constant(1_000), &true);
}
