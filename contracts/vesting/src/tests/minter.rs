use soroban_sdk::{
    testutils::{Address as _, Ledger},
    vec, Address, Env, String,
};

use crate::{
    contract::{Vesting, VestingClient},
    storage::{MinterInfo, VestingSchedule, VestingTokenInfo},
};
use curve::{Curve, SaturatingLinear};

use super::setup::deploy_token_contract;

#[test]
fn instantiate_contract_successfully_with_constant_curve_minter_info() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_client = deploy_token_contract(&env, &admin);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: token_client.address.clone(),
    };

    let minter_info = MinterInfo {
        address: Address::generate(&env),
        mint_capacity: 511223344,
    };

    let vesting_client = VestingClient::new(
        &env,
        &env.register(
            Vesting,
            (&admin, vesting_token.clone(), &10u32, minter_info),
        ),
    );

    token_client.mint(&admin, &240);

    assert_eq!(vesting_client.query_token_info(), vesting_token);
}

#[should_panic(expected = "Vesting: Mint: Minter does not have enough capacity to mint")]
#[test]
fn mint_panics_when_over_the_cap() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    let token_client = deploy_token_contract(&env, &admin);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: token_client.address.clone(),
    };

    let minter = Address::generate(&env);
    let minter_info = MinterInfo {
        address: minter.clone(),
        mint_capacity: 100,
    };

    let vesting_client = VestingClient::new(
        &env,
        &env.register(Vesting, (&admin, vesting_token, &10u32, minter_info)),
    );

    vesting_client.mint(&minter, &110i128);
}

#[test]
fn burn_works() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);
    let token = deploy_token_contract(&env, &admin);

    token.mint(&admin, &1_000);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Token"),
        symbol: String::from_str(&env, "TOK"),
        decimals: 6,
        address: token.address.clone(),
    };
    let vesting_schedules = vec![
        &env,
        VestingSchedule {
            recipient: vester1.clone(),
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
        mint_capacity: 10_000,
    };
    let vesting_client = VestingClient::new(
        &env,
        &env.register(Vesting, (&admin, vesting_token, &10u32, minter_info)),
    );

    vesting_client.create_vesting_schedules(&vesting_schedules);

    env.ledger().with_mut(|li| li.timestamp = 100);
    assert_eq!(vesting_client.query_vesting_contract_balance(), 120);

    vesting_client.claim(&vester1, &0);
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
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let token = deploy_token_contract(&env, &admin);

    token.mint(&admin, &1_000);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Token"),
        symbol: String::from_str(&env, "TOK"),
        decimals: 6,
        address: token.address.clone(),
    };

    let minter_info = MinterInfo {
        address: Address::generate(&env),
        mint_capacity: 10_000,
    };
    let vesting_client = VestingClient::new(
        &env,
        &env.register(Vesting, (&admin, vesting_token, &10u32, minter_info)),
    );

    vesting_client.burn(&Address::generate(&env), &0);
}

#[test]
fn mint_works() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);
    let minter = Address::generate(&env);

    let token = deploy_token_contract(&env, &admin);
    token.mint(&admin, &120);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Token"),
        symbol: String::from_str(&env, "TOK"),
        decimals: 6,
        address: token.address.clone(),
    };
    let vesting_schedules = vec![
        &env,
        VestingSchedule {
            recipient: vester1.clone(),
            curve: Curve::SaturatingLinear(SaturatingLinear {
                min_x: 15,
                min_y: 120,
                max_x: 60,
                max_y: 0,
            }),
        },
    ];

    let minter_info = MinterInfo {
        address: minter.clone(),
        mint_capacity: 500,
    };
    let vesting_client = VestingClient::new(
        &env,
        &env.register(Vesting, (&admin, vesting_token, &10u32, minter_info)),
    );
    vesting_client.create_vesting_schedules(&vesting_schedules);

    // we start with 120 tokens minted to the contract
    assert_eq!(vesting_client.query_vesting_contract_balance(), 120);
    // amdin should have none
    assert_eq!(token.balance(&admin), 0);

    // minter can mint up to 500 tokens
    assert_eq!(vesting_client.query_minter().mint_capacity, 500);

    // user withdraws 120 tokens
    env.ledger().with_mut(|li| li.timestamp = 100);
    vesting_client.claim(&vester1, &0);
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
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);

    let token = deploy_token_contract(&env, &admin);
    token.mint(&admin, &120);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Token"),
        symbol: String::from_str(&env, "TOK"),
        decimals: 6,
        address: token.address.clone(),
    };

    let minter_info = MinterInfo {
        address: admin.clone(),
        mint_capacity: 500,
    };
    let vesting_client = VestingClient::new(
        &env,
        &env.register(Vesting, (&admin, vesting_token, &10u32, minter_info)),
    );

    vesting_client.mint(&Address::generate(&env), &0);
}

#[test]
#[should_panic(expected = "Vesting: Mint: Not authorized to mint")]
fn mint_should_panic_when_not_authorized_to_mint() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);

    let token = deploy_token_contract(&env, &admin);
    token.mint(&admin, &120);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Token"),
        symbol: String::from_str(&env, "TOK"),
        decimals: 6,
        address: token.address.clone(),
    };

    let minter_info = MinterInfo {
        address: Address::generate(&env),
        mint_capacity: 500,
    };
    let vesting_client = VestingClient::new(
        &env,
        &env.register(Vesting, (&admin, vesting_token, &10u32, minter_info)),
    );

    vesting_client.mint(&vester1, &100);
}

#[test]
#[should_panic(expected = "Vesting: Mint: Minter does not have enough capacity to mint")]
fn mint_should_panic_when_mintet_does_not_have_enough_capacity() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let minter = Address::generate(&env);

    let token = deploy_token_contract(&env, &admin);
    token.mint(&admin, &120);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Token"),
        symbol: String::from_str(&env, "TOK"),
        decimals: 6,
        address: token.address.clone(),
    };

    let minter_info = MinterInfo {
        address: minter.clone(),
        mint_capacity: 500,
    };

    let vesting_client = VestingClient::new(
        &env,
        &env.register(Vesting, (&admin, vesting_token, &10u32, minter_info)),
    );

    vesting_client.mint(&minter, &1_500);
}

#[test]
fn update_minter_works_correctly() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let minter = Address::generate(&env);
    let new_minter = Address::generate(&env);

    let token = deploy_token_contract(&env, &admin);
    token.mint(&admin, &120);
    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Token"),
        symbol: String::from_str(&env, "TOK"),
        decimals: 6,
        address: token.address.clone(),
    };

    let minter_info = MinterInfo {
        address: minter.clone(),
        mint_capacity: 500,
    };

    let vesting_client = VestingClient::new(
        &env,
        &env.register(
            Vesting,
            (&admin, vesting_token, &10u32, minter_info.clone()),
        ),
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
#[should_panic(expected = "Vesting: Update minter: Not authorized to update minter")]
fn update_minter_fails_when_not_authorized() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let new_minter = Address::generate(&env);

    let token = deploy_token_contract(&env, &admin);
    token.mint(&admin, &120);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Token"),
        symbol: String::from_str(&env, "TOK"),
        decimals: 6,
        address: token.address.clone(),
    };

    let minter_info = MinterInfo {
        address: Address::generate(&env),
        mint_capacity: 500,
    };

    let vesting_client = VestingClient::new(
        &env,
        &env.register(Vesting, (&admin, vesting_token, &10u32, minter_info)),
    );

    let new_minter_info = MinterInfo {
        address: new_minter.clone(),
        mint_capacity: 1_000,
    };

    vesting_client.update_minter(&Address::generate(&env), &new_minter_info.address);
}

#[test]
fn update_minter_capacity_when_replacing_old_capacity() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let minter = Address::generate(&env);

    let token = deploy_token_contract(&env, &admin);
    token.mint(&admin, &120);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Token"),
        symbol: String::from_str(&env, "TOK"),
        decimals: 6,
        address: token.address.clone(),
    };

    let minter_info = MinterInfo {
        address: minter.clone(),
        mint_capacity: 50_000,
    };

    let vesting_client = VestingClient::new(
        &env,
        &env.register(Vesting, (&admin, vesting_token, &10u32, minter_info)),
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
fn updating_minter_capacity_without_auth() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let minter = Address::generate(&env);

    let token = deploy_token_contract(&env, &admin);
    token.mint(&admin, &120);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Token"),
        symbol: String::from_str(&env, "TOK"),
        decimals: 6,
        address: token.address.clone(),
    };

    let minter_info = MinterInfo {
        address: minter,
        mint_capacity: 50_000,
    };

    let vesting_client = VestingClient::new(
        &env,
        &env.register(Vesting, (&admin, vesting_token, &10u32, minter_info)),
    );

    vesting_client.update_minter_capacity(&Address::generate(&env), &1_000);
}

#[test]
#[should_panic(expected = "zero balance is not sufficient to spend")]
fn burning_more_than_balance() {
    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);

    let token = deploy_token_contract(&env, &admin);
    token.mint(&admin, &120);

    let vesting_token = VestingTokenInfo {
        name: String::from_str(&env, "Token"),
        symbol: String::from_str(&env, "TOK"),
        decimals: 6,
        address: token.address.clone(),
    };

    let minter_info = MinterInfo {
        address: Address::generate(&env),
        mint_capacity: 1_000,
    };
    let vesting_client = VestingClient::new(
        &env,
        &env.register(Vesting, (&admin, vesting_token, &10u32, minter_info)),
    );

    // vester1 tries to burn 121 tokens
    vesting_client.burn(&Address::generate(&env), &121);
}
