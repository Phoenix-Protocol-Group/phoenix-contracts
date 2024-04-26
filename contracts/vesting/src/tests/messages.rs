use soroban_sdk::{testutils::Address as _, vec, Address, Env, String};

use crate::storage::{DistributionInfo, MinterInfo, VestingBalance, VestingTokenInfo};

use super::setup::{deploy_token_contract, instantiate_vesting_client};

#[test]
fn burn_works() {
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
            rcpt_address: vester1.clone(),
            distribution_info: DistributionInfo {
                start_timestamp: 15,
                end_timestamp: 60,
                amount: 120,
            },
        },
    ];

    let vesting_client = instantiate_vesting_client(&env);
    vesting_client.initialize(&admin, &vesting_token, &vesting_balances, &None, &10u32);

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
            rcpt_address: vester1.clone(),
            distribution_info: DistributionInfo {
                start_timestamp: 15,
                end_timestamp: 60,
                amount: 120,
            },
        },
    ];

    let vesting_client = instantiate_vesting_client(&env);
    vesting_client.initialize(&admin, &vesting_token, &vesting_balances, &None, &10u32);

    vesting_client.burn(&vester1, &0);
}

#[test]
#[should_panic(expected = "Vesting: Burn: Critical error - total supply cannot be negative")]
fn burn_should_panic_when_total_supply_becomes_negative() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);
    let token = deploy_token_contract(&env, &admin);

    token.mint(&vester1, &1_300);

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
            rcpt_address: vester1.clone(),
            distribution_info: DistributionInfo {
                start_timestamp: 15,
                end_timestamp: 60,
                amount: 120,
            },
        },
    ];
    let vesting_client = instantiate_vesting_client(&env);
    vesting_client.initialize(&admin, &vesting_token, &vesting_balances, &None, &10u32);
    vesting_client.burn(&vester1, &600);
}

#[test]
fn mint_works() {
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
            rcpt_address: vester1.clone(),
            distribution_info: DistributionInfo {
                start_timestamp: 15,
                end_timestamp: 60,
                amount: 120,
            },
        },
    ];

    let minter_info = MinterInfo {
        address: vester1.clone(),
        mint_capacity: 500,
    };

    let vesting_client = instantiate_vesting_client(&env);
    vesting_client.initialize(
        &admin,
        &vesting_token,
        &vesting_balances,
        &Some(minter_info),
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
            rcpt_address: vester1.clone(),
            distribution_info: DistributionInfo {
                start_timestamp: 15,
                end_timestamp: 60,
                amount: 120,
            },
        },
    ];

    let minter_info = MinterInfo {
        address: vester1.clone(),
        mint_capacity: 500,
    };

    let vesting_client = instantiate_vesting_client(&env);
    vesting_client.initialize(
        &admin,
        &vesting_token,
        &vesting_balances,
        &Some(minter_info),
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
            rcpt_address: vester1.clone(),
            distribution_info: DistributionInfo {
                start_timestamp: 15,
                end_timestamp: 60,
                amount: 120,
            },
        },
    ];

    let minter_info = MinterInfo {
        address: Address::generate(&env),
        mint_capacity: 500,
    };

    let vesting_client = instantiate_vesting_client(&env);
    vesting_client.initialize(
        &admin,
        &vesting_token,
        &vesting_balances,
        &Some(minter_info),
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
            rcpt_address: vester1.clone(),
            distribution_info: DistributionInfo {
                start_timestamp: 15,
                end_timestamp: 60,
                amount: 120,
            },
        },
    ];

    let minter_info = MinterInfo {
        address: vester1.clone(),
        mint_capacity: 500,
    };

    let vesting_client = instantiate_vesting_client(&env);
    vesting_client.initialize(
        &admin,
        &vesting_token,
        &vesting_balances,
        &Some(minter_info),
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
            rcpt_address: vester1.clone(),
            distribution_info: DistributionInfo {
                start_timestamp: 15,
                end_timestamp: 60,
                amount: 120,
            },
        },
    ];

    let minter_info = MinterInfo {
        address: vester1.clone(),
        mint_capacity: 500,
    };

    let vesting_client = instantiate_vesting_client(&env);
    vesting_client.initialize(
        &admin,
        &vesting_token,
        &vesting_balances,
        &Some(minter_info),
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
            rcpt_address: vester1.clone(),
            distribution_info: DistributionInfo {
                start_timestamp: 15,
                end_timestamp: 60,
                amount: 120,
            },
        },
    ];

    let minter_info = MinterInfo {
        address: vester1.clone(),
        mint_capacity: 500,
    };

    let vesting_client = instantiate_vesting_client(&env);
    vesting_client.initialize(
        &admin,
        &vesting_token,
        &vesting_balances,
        &Some(minter_info.clone()),
        &10u32,
    );

    assert_eq!(vesting_client.query_minter(), minter_info);

    let new_minter_info = MinterInfo {
        address: new_minter.clone(),
        mint_capacity: 1_000,
    };

    vesting_client.update_minter(&vester1, &new_minter_info.address);

    assert_eq!(
        vesting_client.query_minter().address,
        new_minter_info.address
    );
}

#[test]
fn update_minter_works_correctly_when_no_minter_was_set_initially() {
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
            rcpt_address: vester1.clone(),
            distribution_info: DistributionInfo {
                start_timestamp: 15,
                end_timestamp: 60,
                amount: 120,
            },
        },
    ];

    let vesting_client = instantiate_vesting_client(&env);
    vesting_client.initialize(&admin, &vesting_token, &vesting_balances, &None, &10u32);

    let new_minter_info = MinterInfo {
        address: new_minter.clone(),
        mint_capacity: 1_000,
    };

    vesting_client.update_minter(&admin, &new_minter_info.address);

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
            rcpt_address: vester1.clone(),
            distribution_info: DistributionInfo {
                start_timestamp: 15,
                end_timestamp: 60,
                amount: 120,
            },
        },
    ];

    let minter_info = MinterInfo {
        address: Address::generate(&env),
        mint_capacity: 500,
    };

    let vesting_client = instantiate_vesting_client(&env);
    vesting_client.initialize(
        &admin,
        &vesting_token,
        &vesting_balances,
        &Some(minter_info.clone()),
        &10u32,
    );

    let new_minter_info = MinterInfo {
        address: new_minter.clone(),
        mint_capacity: 1_000,
    };

    vesting_client.update_minter(&Address::generate(&env), &new_minter_info.address);
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
            rcpt_address: vester1.clone(),
            distribution_info: DistributionInfo {
                start_timestamp: 15,
                end_timestamp: 60,
                amount: 120,
            },
        },
    ];

    let minter_info = MinterInfo {
        address: vester1.clone(),
        mint_capacity: 500,
    };

    let vesting_client = instantiate_vesting_client(&env);
    vesting_client.initialize(
        &admin,
        &vesting_token,
        &vesting_balances,
        &Some(minter_info.clone()),
        &10u32,
    );

    let new_minter_capacity = 1_000;
    vesting_client.update_minter_capacity(&admin, &new_minter_capacity);

    assert_eq!(
        vesting_client.query_minter().mint_capacity,
        new_minter_capacity
    );
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
            rcpt_address: vester1.clone(),
            distribution_info: DistributionInfo {
                start_timestamp: 15,
                end_timestamp: 60,
                amount: 120,
            },
        },
    ];

    let minter_info = MinterInfo {
        address: vester1.clone(),
        mint_capacity: 500,
    };

    let vesting_client = instantiate_vesting_client(&env);
    vesting_client.initialize(
        &admin,
        &vesting_token,
        &vesting_balances,
        &Some(minter_info.clone()),
        &10u32,
    );

    let new_capacity = 1_000;
    vesting_client.update_minter_capacity(&admin, &new_capacity);

    assert_eq!(vesting_client.query_minter().mint_capacity, 1_000);
}

#[test]
#[should_panic(
    expected = "Vesting: Update minter capacity: Only contract's admin can update the minter's capacity"
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
            rcpt_address: vester1.clone(),
            distribution_info: DistributionInfo {
                start_timestamp: 15,
                end_timestamp: 60,
                amount: 120,
            },
        },
    ];

    let minter_info = MinterInfo {
        address: vester1.clone(),
        mint_capacity: 500,
    };

    let vesting_client = instantiate_vesting_client(&env);
    vesting_client.initialize(
        &admin,
        &vesting_token,
        &vesting_balances,
        &Some(minter_info.clone()),
        &10u32,
    );

    vesting_client.update_minter_capacity(&Address::generate(&env), &1_000);
}
