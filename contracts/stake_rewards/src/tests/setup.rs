use soroban_sdk::{Address, BytesN, Env};

use crate::{
    contract::{StakingRewards, StakingRewardsClient},
    token_contract,
};

#[allow(clippy::too_many_arguments)]
pub mod old_stake_rewards {
    soroban_sdk::contractimport!(file = "../../.artifacts/old_phoenix_stake_rewards.wasm");
}

#[allow(clippy::too_many_arguments)]
pub mod latest_stake_rewards {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/phoenix_stake_rewards.wasm"
    );
}

#[allow(clippy::too_many_arguments)]
pub fn install_stake_rewards_wasm(env: &Env) -> BytesN<32> {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/phoenix_stake_rewards.wasm"
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

const MIN_BOND: i128 = 1000;
const MIN_REWARD: i128 = 1000;
const MAX_COMPLEXITY: u32 = 10;

pub fn deploy_staking_rewards_contract<'a>(
    env: &Env,
    admin: &Address,
    reward_token: &Address,
    staking_contract: &Address,
) -> StakingRewardsClient<'a> {
    let staking_rewards = StakingRewardsClient::new(env, &env.register(StakingRewards, ()));

    staking_rewards.initialize(
        admin,
        staking_contract,
        reward_token,
        &MAX_COMPLEXITY,
        &MIN_REWARD,
        &MIN_BOND,
    );
    staking_rewards
}

#[test]
#[allow(deprecated)]
#[cfg(feature = "upgrade")]
fn updapte_stake_rewards() {
    use soroban_sdk::testutils::Address as _;

    let env = Env::default();
    env.mock_all_auths();
    env.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&env);
    let staking_contract = Address::generate(&env);
    let reward_token = Address::generate(&env);

    let old_stake_rewards_addr = env.register_contract_wasm(None, old_stake_rewards::WASM);

    let old_stake_rewards_client = old_stake_rewards::Client::new(&env, &old_stake_rewards_addr);

    old_stake_rewards_client.initialize(
        &admin,
        &staking_contract,
        &reward_token,
        &MAX_COMPLEXITY,
        &MIN_REWARD,
        &MIN_BOND,
    );

    assert_eq!(old_stake_rewards_client.query_admin(), admin);
    assert_eq!(
        old_stake_rewards_client.query_config().config,
        old_stake_rewards::Config {
            max_complexity: MAX_COMPLEXITY,
            min_bond: MIN_BOND,
            min_reward: MIN_REWARD,
            reward_token: reward_token.clone(),
            staking_contract: staking_contract.clone(),
        }
    );

    let latest_stake_rewards_wasm = install_stake_rewards_wasm(&env);

    old_stake_rewards_client.update(&latest_stake_rewards_wasm);

    let latest_stake_rewards_client =
        latest_stake_rewards::Client::new(&env, &old_stake_rewards_addr);

    assert_eq!(latest_stake_rewards_client.query_admin(), admin);
    assert_eq!(
        latest_stake_rewards_client.query_config().config,
        latest_stake_rewards::Config {
            max_complexity: MAX_COMPLEXITY,
            min_bond: MIN_BOND,
            min_reward: MIN_REWARD,
            reward_token,
            staking_contract
        }
    );
}
