use soroban_sdk::{testutils::Address as _, Address, Env};

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

const MIN_BOND: i128 = 1000;
const MIN_REWARD: i128 = 1000;

pub fn deploy_staking_contract<'a>(
    env: &Env,
    admin: impl Into<Option<Address>>,
    lp_token: &Address,
    manager: &Address,
    owner: &Address,
    max_complexity: &u32,
) -> StakingClient<'a> {
    let admin = admin.into().unwrap_or(Address::generate(env));
    let staking = StakingClient::new(env, &env.register(Staking, ()));

    staking.initialize(
        &admin,
        lp_token,
        &MIN_BOND,
        &MIN_REWARD,
        manager,
        owner,
        max_complexity,
    );
    staking
}

#[cfg(test)]
#[allow(clippy::too_many_arguments)]
mod tests {

    pub mod token {
        // The import will code generate:
        // - A ContractClient type that can be used to invoke functions on the contract.
        // - Any types in the contract that were annotated with #[contracttype].
        soroban_sdk::contractimport!(
            file = "../../target/wasm32-unknown-unknown/release/soroban_token_contract.wasm"
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub mod old_stake {
        soroban_sdk::contractimport!(
            file = "../../.artifacts_stake_migration_test/old_phoenix_stake.wasm"
        );
    }

    use old_stake::StakedResponse;
    use pretty_assertions::assert_eq;
    use soroban_sdk::testutils::Ledger;
    use soroban_sdk::{testutils::Address as _, Address};
    use soroban_sdk::{vec, Env, String};

    #[test]
    fn upgrade_staking_contract_and_remove_stake_rewards() {
        const DAY_AS_SECONDS: u64 = 86_400;

        let env = Env::default();
        env.mock_all_auths();
        env.cost_estimate().budget().reset_unlimited();
        let admin = Address::generate(&env);
        let manager = Address::generate(&env);
        let owner = Address::generate(&env);
        let user_1 = Address::generate(&env);
        let user_2 = Address::generate(&env);
        let user_3 = Address::generate(&env);

        let factory_addr = env.register(old_stake::WASM, ());
        let old_stake_client = old_stake::Client::new(&env, &factory_addr);

        let lp_token_addr = env.register(
            token::WASM,
            (
                Address::generate(&env),
                7u32,
                String::from_str(&env, "LP Token"),
                String::from_str(&env, "LPT"),
            ),
        );

        let lp_token_client = token::Client::new(&env, &lp_token_addr);
        lp_token_client.mint(&user_1, &10_000_000_000_000);
        lp_token_client.mint(&user_2, &10_000_000_000_000);
        lp_token_client.mint(&user_3, &10_000_000_000_000);

        let reward_token_addr = env.register(
            token::WASM,
            (
                Address::generate(&env),
                7u32,
                String::from_str(&env, "Reward Token"),
                String::from_str(&env, "RWT"),
            ),
        );

        let reward_token_client = token::Client::new(&env, &reward_token_addr);
        reward_token_client.mint(&manager, &10_000_000_000_000);

        old_stake_client.initialize(
            &admin,
            &lp_token_client.address,
            &100,
            &50,
            &manager,
            &owner,
            &7,
        );

        // after a day the manager creates a distribution flow
        env.ledger().with_mut(|li| li.timestamp += DAY_AS_SECONDS);
        old_stake_client.create_distribution_flow(&manager, &reward_token_addr);

        // another day passes and the users bond
        env.ledger().with_mut(|li| li.timestamp += DAY_AS_SECONDS);
        old_stake_client.bond(&user_1, &10_000_000_000); // user_1 bonds 1,000 tokens
        old_stake_client.bond(&user_2, &20_000_000_000); // user_2 bonds 2,000 tokens
        old_stake_client.bond(&user_3, &15_000_000_000); // user_3 bonds 1,500 tokens

        // Assert staked amounts for all users
        assert_eq!(
            old_stake_client.query_staked(&user_1),
            StakedResponse {
                last_reward_time: 0,
                stakes: vec![
                    &env,
                    old_stake::Stake {
                        stake: 10_000_000_000,
                        stake_timestamp: DAY_AS_SECONDS * 2,
                    }
                ],
                total_stake: 10_000_000_000
            }
        );

        assert_eq!(
            old_stake_client.query_staked(&user_2),
            StakedResponse {
                last_reward_time: 0,
                stakes: vec![
                    &env,
                    old_stake::Stake {
                        stake: 20_000_000_000,
                        stake_timestamp: DAY_AS_SECONDS * 2,
                    }
                ],
                total_stake: 20_000_000_000
            }
        );

        assert_eq!(
            old_stake_client.query_staked(&user_3),
            StakedResponse {
                last_reward_time: 0,
                stakes: vec![
                    &env,
                    old_stake::Stake {
                        stake: 15_000_000_000,
                        stake_timestamp: DAY_AS_SECONDS * 2,
                    }
                ],
                total_stake: 15_000_000_000
            }
        );

        // 100 days forward after staking let's check the rewards
        env.ledger()
            .with_mut(|li| li.timestamp += 100 * DAY_AS_SECONDS);

        let user_1_withdrawable_rewards = old_stake_client.query_withdrawable_rewards(&user_1);
        let user_2_withdrawable_rewards = old_stake_client.query_withdrawable_rewards(&user_2);
        let user_3_withdrawable_rewards = old_stake_client.query_withdrawable_rewards(&user_3);

        old_stake_client.distribute_rewards(&manager, &100, &reward_token_addr);

        soroban_sdk::testutils::arbitrary::std::dbg!(
            user_1_withdrawable_rewards,
            user_2_withdrawable_rewards,
            user_3_withdrawable_rewards
        );
    }
}
