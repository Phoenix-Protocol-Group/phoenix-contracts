use soroban_sdk::{
    testutils::{Address as _, Ledger},
    vec, Address, Env, String,
};

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

    token.mint(&admin, &1_000);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: token.address.clone(),
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

    env.ledger().with_mut(|li| li.timestamp = 100);
    assert_eq!(vesting_client.query_vesting_contract_balance(), 120);

    vesting_client.transfer_token(&vester1, &vester1, &120);
    assert_eq!(vesting_client.query_vesting_contract_balance(), 0);
    assert_eq!(token.balance(&vester1), 120);

    vesting_client.burn(&vester1, &120);
    assert_eq!(token.balance(&vester1), 0);
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

    token.mint(&admin, &1_000);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: token.address.clone(),
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
fn mint_works() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);
    let minter = Address::generate(&env);

    let token = deploy_token_contract(&env, &admin);
    token.mint(&admin, &120);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: token.address.clone(),
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
        address: minter.clone(),
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

    // we start with 120 tokens minted to the contract
    assert_eq!(vesting_client.query_vesting_contract_balance(), 120);
    // amdin should have none
    assert_eq!(token.balance(&admin), 0);

    // minter can mint up to 500 tokens
    assert_eq!(vesting_client.query_minter().mint_capacity, 500);

    // user withdraws 120 tokens
    env.ledger().with_mut(|li| li.timestamp = 100);
    vesting_client.transfer_token(&vester1, &vester1, &120);
    assert_eq!(token.balance(&vester1), 120);
    assert_eq!(vesting_client.query_vesting_contract_balance(), 0);

    // minter decides to mint new 250 tokens
    vesting_client.mint(&minter, &250);
    assert_eq!(vesting_client.query_vesting_contract_balance(), 250);
    assert_eq!(vesting_client.query_minter().mint_capacity, 250);

    // we mint 250 more tokens
    vesting_client.mint(&minter, &250);
    assert_eq!(vesting_client.query_vesting_contract_balance(), 500);
    assert_eq!(vesting_client.query_minter().mint_capacity, 0);
}

#[test]
#[should_panic(expected = "Vesting: Mint: Invalid mint amount")]
fn mint_should_panic_when_invalid_amount() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);

    let token = deploy_token_contract(&env, &admin);
    token.mint(&admin, &120);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: token.address.clone(),
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

    vesting_client.mint(&vester1, &0);
}

#[test]
#[should_panic(expected = "Vesting: Mint: Not authorized to mint")]
fn mint_should_panic_when_not_authorized_to_mint() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);

    let token = deploy_token_contract(&env, &admin);
    token.mint(&admin, &120);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: token.address.clone(),
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

    vesting_client.mint(&vester1, &100);
}

#[test]
#[should_panic(expected = "Vesting: Mint: Minter does not have enough capacity to mint")]
fn mint_should_panic_when_mintet_does_not_have_enough_capacity() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let minter = Address::generate(&env);
    let vester1 = Address::generate(&env);

    let token = deploy_token_contract(&env, &admin);
    token.mint(&admin, &120);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: token.address.clone(),
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
        address: minter.clone(),
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

    vesting_client.mint(&minter, &1_500);
}

#[test]
fn update_minter_works_correctly() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);
    let minter = Address::generate(&env);
    let new_minter = Address::generate(&env);

    let token = deploy_token_contract(&env, &admin);
    token.mint(&admin, &120);
    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: token.address.clone(),
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
        address: minter.clone(),
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

    vesting_client.update_minter(&minter, &new_minter_info.address);

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
    token.mint(&admin, &120);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: token.address.clone(),
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
    token.mint(&admin, &120);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: token.address.clone(),
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
#[should_panic(expected = "Vesting: Mint: Minter not found")]
fn minting_fails_because_no_minter_was_found() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);

    let token = deploy_token_contract(&env, &admin);
    token.mint(&admin, &120);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: token.address.clone(),
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

    vesting_client.mint(&Address::generate(&env), &500);
}

#[test]
#[should_panic(expected = "Vesting: Update Minter Capacity: Minter not found")]
fn update_minter_fails_because_no_minter_found() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);

    let token = deploy_token_contract(&env, &admin);
    token.mint(&admin, &120);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: token.address.clone(),
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

    vesting_client.update_minter_capacity(&admin, &500);
}

#[test]
#[should_panic(expected = "Vesting: Query Minter: Minter not found")]
fn query_minter_should_fail_because_no_minter_found() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);

    let token = deploy_token_contract(&env, &admin);
    token.mint(&admin, &120);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: token.address.clone(),
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

    vesting_client.query_minter();
}

#[test]
fn test_should_update_minter_capacity_when_replacing_old_capacity() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);
    let minter = Address::generate(&env);

    let token = deploy_token_contract(&env, &admin);
    token.mint(&admin, &120);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: token.address.clone(),
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
        address: minter.clone(),
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
#[should_panic(
    expected = "Vesting: Update minter capacity: Only contract's admin can update the minter's capacity"
)]
fn test_should_panic_when_updating_minter_capacity_without_auth() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);
    let minter = Address::generate(&env);

    let token = deploy_token_contract(&env, &admin);
    token.mint(&admin, &120);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: token.address.clone(),
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
        address: minter,
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

#[test]
#[should_panic(expected = "zero balance is not sufficient to spend")]
fn test_should_fail_when_burning_more_than_the_user_has() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);

    let token = deploy_token_contract(&env, &admin);
    token.mint(&admin, &120);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: token.address.clone(),
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

    // vester1 has 0 tokens
    assert_eq!(token.balance(&vester1), 0);
    // vester1 tries to burn 121 tokens
    vesting_client.burn(&vester1, &121);
}
