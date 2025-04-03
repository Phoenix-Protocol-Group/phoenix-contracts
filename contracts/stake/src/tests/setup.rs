use crate::{
    contract::{Staking, StakingClient},
    token_contract,
};
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};

pub fn deploy_token_contract<'a>(env: &Env, admin: &Address) -> token_contract::Client<'a> {
    token_contract::Client::new(
        env,
        &env.register_stellar_asset_contract_v2(admin.clone())
            .address(),
    )
}

#[allow(clippy::too_many_arguments)]
pub fn install_stake_wasm(env: &Env) -> BytesN<32> {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/phoenix_stake.wasm"
    );
    env.deployer().upload_contract_wasm(WASM)
}

#[allow(clippy::too_many_arguments)]
pub mod latest_stake {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/phoenix_stake.wasm"
    );
}

#[allow(clippy::too_many_arguments)]
#[cfg(feature = "upgrade")]
mod old_stake {
    soroban_sdk::contractimport!(file = "../../.artifacts_sdk_update/old_phoenix_stake.wasm");
}

#[allow(dead_code)]
fn install_stake_latest_wasm(env: &Env) -> BytesN<32> {
    env.deployer().upload_contract_wasm(latest_stake::WASM)
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

#[cfg(test)]
#[allow(clippy::too_many_arguments)]
pub mod tests {

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

    use old_stake::{StakedResponse, WithdrawableReward, WithdrawableRewardsResponse};
    use pretty_assertions::assert_eq;
    use soroban_sdk::testutils::Ledger;
    use soroban_sdk::{testutils::Address as _, Address};
    use soroban_sdk::{vec, Env, String};

    use crate::tests::setup::{install_stake_wasm, latest_stake};

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

        let new_user = Address::generate(&env);

        let stake_addr = env.register(old_stake::WASM, ());
        let old_stake_client = old_stake::Client::new(&env, &stake_addr);

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
        reward_token_client.mint(&manager, &100_000_000_000_000);

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

        // 30 days forward after staking let's check the rewards
        env.ledger()
            .with_mut(|li| li.timestamp += 30 * DAY_AS_SECONDS);

        assert_eq!(
            old_stake_client.query_withdrawable_rewards(&user_1),
            WithdrawableRewardsResponse {
                rewards: vec![
                    &env,
                    WithdrawableReward {
                        reward_address: reward_token_addr.clone(),
                        reward_amount: 0,
                    }
                ]
            }
        );

        assert_eq!(
            old_stake_client.query_withdrawable_rewards(&user_2),
            WithdrawableRewardsResponse {
                rewards: vec![
                    &env,
                    WithdrawableReward {
                        reward_address: reward_token_addr.clone(),
                        reward_amount: 0,
                    }
                ]
            }
        );

        assert_eq!(
            old_stake_client.query_withdrawable_rewards(&user_3),
            WithdrawableRewardsResponse {
                rewards: vec![
                    &env,
                    WithdrawableReward {
                        reward_address: reward_token_addr.clone(),
                        reward_amount: 0,
                    }
                ]
            }
        );

        old_stake_client.distribute_rewards(&manager, &10_000_000, &reward_token_addr);

        assert_eq!(
            old_stake_client.query_withdrawable_rewards(&user_1),
            WithdrawableRewardsResponse {
                rewards: vec![
                    &env,
                    WithdrawableReward {
                        reward_address: reward_token_addr.clone(),
                        reward_amount: 1_111_111,
                    }
                ]
            }
        );

        assert_eq!(
            old_stake_client.query_withdrawable_rewards(&user_2),
            WithdrawableRewardsResponse {
                rewards: vec![
                    &env,
                    WithdrawableReward {
                        reward_address: reward_token_addr.clone(),
                        reward_amount: 2_222_222,
                    }
                ]
            }
        );

        assert_eq!(
            old_stake_client.query_withdrawable_rewards(&user_3),
            WithdrawableRewardsResponse {
                rewards: vec![
                    &env,
                    WithdrawableReward {
                        reward_address: reward_token_addr.clone(),
                        reward_amount: 1_666_666,
                    }
                ]
            }
        );

        // we upgrade
        let new_stake_wasm = install_stake_wasm(&env);

        old_stake_client.update(&new_stake_wasm);

        let latest_stake_client = latest_stake::Client::new(&env, &stake_addr);

        // now we migrate the distributions
        latest_stake_client.migrate_distributions();

        // check the rewards again, this time with the old deprecated method
        assert_eq!(
            latest_stake_client.query_withdrawable_rewards_dep(&user_1),
            latest_stake::WithdrawableRewardsResponse {
                rewards: vec![
                    &env,
                    latest_stake::WithdrawableReward {
                        reward_address: reward_token_addr.clone(),
                        reward_amount: 1_111_111,
                    }
                ]
            }
        );

        assert_eq!(
            latest_stake_client.query_withdrawable_rewards_dep(&user_2),
            latest_stake::WithdrawableRewardsResponse {
                rewards: vec![
                    &env,
                    latest_stake::WithdrawableReward {
                        reward_address: reward_token_addr.clone(),
                        reward_amount: 2_222_222,
                    }
                ]
            }
        );

        assert_eq!(
            latest_stake_client.query_withdrawable_rewards_dep(&user_3),
            latest_stake::WithdrawableRewardsResponse {
                rewards: vec![
                    &env,
                    latest_stake::WithdrawableReward {
                        reward_address: reward_token_addr.clone(),
                        reward_amount: 1_666_666,
                    }
                ]
            }
        );

        latest_stake_client.withdraw_rewards_deprecated(&user_1);
        latest_stake_client.withdraw_rewards_deprecated(&user_2);
        latest_stake_client.withdraw_rewards_deprecated(&user_3);

        // we make sure that there are no more rewards
        assert_eq!(
            latest_stake_client.query_withdrawable_rewards_dep(&user_1),
            latest_stake::WithdrawableRewardsResponse {
                rewards: vec![
                    &env,
                    latest_stake::WithdrawableReward {
                        reward_address: reward_token_addr.clone(),
                        reward_amount: 0,
                    }
                ]
            }
        );

        assert_eq!(
            latest_stake_client.query_withdrawable_rewards_dep(&user_2),
            latest_stake::WithdrawableRewardsResponse {
                rewards: vec![
                    &env,
                    latest_stake::WithdrawableReward {
                        reward_address: reward_token_addr.clone(),
                        reward_amount: 0,
                    }
                ]
            }
        );

        assert_eq!(
            latest_stake_client.query_withdrawable_rewards_dep(&user_3),
            latest_stake::WithdrawableRewardsResponse {
                rewards: vec![
                    &env,
                    latest_stake::WithdrawableReward {
                        reward_address: reward_token_addr.clone(),
                        reward_amount: 0,
                    }
                ]
            }
        );

        assert_eq!(reward_token_client.balance(&user_1), 1_111_111);
        assert_eq!(reward_token_client.balance(&user_2), 2_222_222);
        assert_eq!(reward_token_client.balance(&user_3), 1_666_666);

        // query the staked before unbonding
        assert_eq!(
            latest_stake_client.query_staked(&user_1),
            latest_stake::StakedResponse {
                stakes: vec![
                    &env,
                    latest_stake::Stake {
                        stake: 10000000000,
                        stake_timestamp: DAY_AS_SECONDS * 2
                    }
                ]
            }
        );

        assert_eq!(
            latest_stake_client.query_staked(&user_2),
            latest_stake::StakedResponse {
                stakes: vec![
                    &env,
                    latest_stake::Stake {
                        stake: 20_000_000_000,
                        stake_timestamp: DAY_AS_SECONDS * 2
                    }
                ]
            }
        );

        assert_eq!(
            latest_stake_client.query_staked(&user_3),
            latest_stake::StakedResponse {
                stakes: vec![
                    &env,
                    latest_stake::Stake {
                        stake: 15_000_000_000,
                        stake_timestamp: DAY_AS_SECONDS * 2
                    }
                ]
            }
        );

        // 30 days pass by and this time users directly unbond 1/2 which should also get their
        //    rewards
        env.ledger()
            .with_mut(|li| li.timestamp += 30 * DAY_AS_SECONDS);

        latest_stake_client.unbond_deprecated(&user_1, &10000000000, &(172800));
        latest_stake_client.unbond_deprecated(&user_2, &20000000000, &(172800));
        latest_stake_client.unbond_deprecated(&user_3, &15000000000, &(172800));

        assert_eq!(
            latest_stake_client.query_staked(&user_1),
            latest_stake::StakedResponse {
                stakes: vec![&env,]
            }
        );

        assert_eq!(
            latest_stake_client.query_staked(&user_2),
            latest_stake::StakedResponse {
                stakes: vec![&env,]
            }
        );

        assert_eq!(
            latest_stake_client.query_staked(&user_3),
            latest_stake::StakedResponse {
                stakes: vec![&env,]
            }
        );

        // one more day passes by and new_user decides to stake
        env.ledger().with_mut(|li| li.timestamp += DAY_AS_SECONDS);

        lp_token_client.mint(&new_user, &10_000_000_000_000);

        let time_of_bond = env.ledger().timestamp();
        latest_stake_client.bond(&new_user, &10_000_000_000); // new_user also bonds 1,000 tokens

        // two months pass by
        env.ledger()
            .with_mut(|li| li.timestamp += 60 * DAY_AS_SECONDS);

        // distribute and take the rewards
        latest_stake_client.distribute_rewards();

        assert_eq!(
            latest_stake_client.query_withdrawable_rewards(&new_user),
            latest_stake::WithdrawableRewardsResponse {
                rewards: vec![
                    &env,
                    latest_stake::WithdrawableReward {
                        reward_address: reward_token_addr.clone(),
                        reward_amount: 5_000_000,
                    }
                ]
            }
        );

        latest_stake_client.withdraw_rewards(&new_user);
        assert_eq!(reward_token_client.balance(&new_user), 5_000_000);

        latest_stake_client.unbond(&new_user, &10_000_000_000, &time_of_bond);
        assert_eq!(lp_token_client.balance(&new_user), 10_000_000_000_000);
    }

    #[test]
    #[allow(deprecated)]
    #[cfg(feature = "upgrade")]
    fn upgrade_stake_contract() {
        use soroban_sdk::{testutils::Ledger, vec};

        use crate::tests::setup::{deploy_token_contract, install_stake_latest_wasm};

        let env = Env::default();
        env.mock_all_auths();
        env.cost_estimate().budget().reset_unlimited();
        let admin = Address::generate(&env);
        let user = Address::generate(&env);

        let token_client = deploy_token_contract(&env, &admin);
        token_client.mint(&user, &1_000);

        let stake_addr = env.register_contract_wasm(None, old_stake::WASM);

        let old_stake_client = old_stake::Client::new(&env, &stake_addr);

        let manager = Address::generate(&env);
        let owner = Address::generate(&env);

        old_stake_client.initialize(
            &admin,
            &token_client.address,
            &10,
            &10,
            &manager,
            &owner,
            &10,
        );

        token_client.mint(&owner, &1_000);
        old_stake_client.create_distribution_flow(&owner, &token_client.address);

        assert_eq!(old_stake_client.query_admin(), admin);

        env.ledger().with_mut(|li| li.timestamp = 100);
        old_stake_client.bond(&user, &1_000);
        assert_eq!(
            old_stake_client.query_staked(&user),
            old_stake::StakedResponse {
                stakes: vec![
                    &env,
                    old_stake::Stake {
                        stake: 1_000i128,
                        stake_timestamp: 100
                    }
                ],
                last_reward_time: 0u64,
                total_stake: 1_000i128,
            }
        );

        env.ledger().with_mut(|li| li.timestamp = 10_000);

        let new_stake_wasm = install_stake_latest_wasm(&env);
        old_stake_client.update(&new_stake_wasm);

        let new_stake_client = latest_stake::Client::new(&env, &stake_addr);
        new_stake_client.migrate_distributions();

        assert_eq!(new_stake_client.query_admin(), admin);

        env.ledger().with_mut(|li| li.timestamp = 20_000);
        new_stake_client.distribute_rewards();

        soroban_sdk::testutils::arbitrary::std::dbg!(
            new_stake_client.query_withdrawable_rewards(&user)
        );

        new_stake_client.unbond_deprecated(&user, &1_000, &100);

        assert_eq!(
            new_stake_client.query_staked(&user),
            latest_stake::StakedResponse {
                stakes: vec![&env,],
            }
        );
    }
}
