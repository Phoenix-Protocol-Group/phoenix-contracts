use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};

use crate::{
    contract::{Staking, StakingClient},
    token_contract,
};

pub fn deploy_token_contract<'a>(env: &Env, admin: &Address) -> token_contract::Client<'a> {
    token_contract::Client::new(
        env,
        &env.register_stellar_asset_contract_v2(admin.clone())
            .address(),
    )
}

#[allow(clippy::too_many_arguments)]
mod stake_latest {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/phoenix_stake.wasm"
    );
}

#[allow(dead_code)]
fn install_stake_latest_wasm(env: &Env) -> BytesN<32> {
    env.deployer().upload_contract_wasm(stake_latest::WASM)
}

const MIN_BOND: i128 = 1000;
const MIN_REWARD: i128 = 1000;
pub const ONE_WEEK: u64 = 604800;
pub const ONE_DAY: u64 = 86400;
pub const SIXTY_DAYS: u64 = 60 * ONE_DAY;

pub fn deploy_staking_contract<'a>(
    env: &Env,
    admin: impl Into<Option<Address>>,
    lp_token: &Address,
    manager: &Address,
    owner: &Address,
    max_complexity: &u32,
) -> StakingClient<'a> {
    let admin = admin.into().unwrap_or(Address::generate(env));
    let staking = StakingClient::new(
        env,
        &env.register(
            Staking,
            (
                &admin,
                lp_token,
                &MIN_BOND,
                &MIN_REWARD,
                manager,
                owner,
                max_complexity,
            ),
        ),
    );

    staking
}

#[cfg(feature = "upgrade")]
use soroban_sdk::{testutils::Ledger, vec};

#[test]
#[cfg(feature = "upgrade")]
fn upgrade_stake_contract() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    let token_client = deploy_token_contract(&env, &admin);
    token_client.mint(&user, &1_000);

    let stake_addr = env.register_contract_wasm(None, stake_v_1_0_0::WASM);

    let stake_v_1_0_0_client = stake_v_1_0_0::Client::new(&env, &stake_addr);

    let manager = Address::generate(&env);
    let owner = Address::generate(&env);

    stake_v_1_0_0_client.initialize(
        &admin,
        &token_client.address,
        &10,
        &10,
        &manager,
        &owner,
        &10,
    );

    assert_eq!(stake_v_1_0_0_client.query_admin(), admin);

    env.ledger().with_mut(|li| li.timestamp = 100);
    stake_v_1_0_0_client.bond(&user, &1_000);
    assert_eq!(
        stake_v_1_0_0_client.query_staked(&user),
        stake_v_1_0_0::StakedResponse {
            stakes: vec![
                &env,
                stake_v_1_0_0::Stake {
                    stake: 1_000i128,
                    stake_timestamp: 100
                }
            ]
        }
    );

    env.ledger().with_mut(|li| li.timestamp = 10_000);

    let new_stake_wasm = install_stake_latest_wasm(&env);
    stake_v_1_0_0_client.update(&new_stake_wasm);
    stake_v_1_0_0_client.update(&new_stake_wasm);

    let upgraded_stake_client = stake_latest::Client::new(&env, &stake_addr);

    assert_eq!(upgraded_stake_client.query_admin(), admin);

    env.ledger().with_mut(|li| li.timestamp = 20_000);

    upgraded_stake_client.unbond(&user, &1_000, &100);
    assert_eq!(
        upgraded_stake_client.query_staked(&user),
        stake_latest::StakedResponse {
            stakes: vec![&env,],
            total_stake: 0i128
        }
    );

    upgraded_stake_client.create_distribution_flow(&owner, &token_client.address);
    token_client.mint(&owner, &1_000);
    upgraded_stake_client.distribute_rewards(&owner, &1_000, &token_client.address);
}
