use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, BytesN, Env,
};

use crate::token_contract;

#[allow(clippy::too_many_arguments)]
pub mod old_vesting {
    soroban_sdk::contractimport!(file = "../../.wasm_binaries_mainnet/live_vesting.wasm");
}

pub fn install_latest_vesting(env: &Env) -> BytesN<32> {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/phoenix_vesting.wasm"
    );
    env.deployer().upload_contract_wasm(WASM)
}

pub fn deploy_token_contract<'a>(env: &Env, admin: &Address) -> token_contract::Client<'a> {
    token_contract::Client::new(
        env,
        &env.register_stellar_asset_contract_v2(admin.clone())
            .address(),
    )
}

#[test]
#[allow(deprecated)]
fn upgrade_vesting_contract() {
    use soroban_sdk::{vec, String};

    use crate::tests::setup::deploy_token_contract;

    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    let token_client = deploy_token_contract(&env, &admin);
    token_client.mint(&user, &1_000);

    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let vester1 = Address::generate(&env);
    let token_client = deploy_token_contract(&env, &admin);

    token_client.mint(&admin, &320);

    let vesting_token_info = old_vesting::VestingTokenInfo {
        name: String::from_str(&env, "Phoenix"),
        symbol: String::from_str(&env, "PHO"),
        decimals: 6,
        address: token_client.address.clone(),
    };

    let vesting_schedules = vec![
        &env,
        old_vesting::VestingSchedule {
            recipient: vester1.clone(),
            curve: old_vesting::Curve::SaturatingLinear(old_vesting::SaturatingLinear {
                min_x: 0,
                min_y: 120,
                max_x: 60,
                max_y: 0,
            }),
        },
    ];

    let vesting_addr = env.register_contract_wasm(None, old_vesting::WASM);

    let old_vesting_client = old_vesting::Client::new(&env, &vesting_addr);

    old_vesting_client.initialize(&admin, &vesting_token_info, &10u32);

    old_vesting_client.create_vesting_schedules(&vesting_schedules);

    assert_eq!(token_client.balance(&old_vesting_client.address), 120);

    env.ledger().with_mut(|li| li.timestamp = 30);

    old_vesting_client.claim(&vester1, &0);
    assert_eq!(token_client.balance(&vester1), 60);
    assert_eq!(token_client.balance(&old_vesting_client.address), 60);

    let new_wasm_hash = install_latest_vesting(&env);
    old_vesting_client.update(&new_wasm_hash);

    assert_eq!(token_client.balance(&vester1), 60);
    assert_eq!(token_client.balance(&old_vesting_client.address), 60);

    // fully vested
    env.ledger().with_mut(|li| li.timestamp = 60);

    old_vesting_client.claim(&vester1, &0);

    assert_eq!(token_client.balance(&vester1), 120);
    assert_eq!(token_client.balance(&old_vesting_client.address), 0);
}
