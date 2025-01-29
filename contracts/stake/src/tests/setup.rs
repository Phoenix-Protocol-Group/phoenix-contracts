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

    const TOKEN_WASM: &[u8] = include_bytes!(
        "../../../../target/wasm32-unknown-unknown/release/soroban_token_contract.wasm"
    );

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
        soroban_sdk::contractimport!(file = "../../artifacts/old_phoenix_stake.wasm");
    }

    use soroban_sdk::{testutils::Address as _, Address};
    use soroban_sdk::{Env, String};

    #[test]
    fn upgrade_staking_contract_and_remove_stake_rewards() {
        let env = Env::default();
        env.mock_all_auths();
        env.cost_estimate().budget().reset_unlimited();
        let admin = Address::generate(&env);
        let manager = Address::generate(&env);
        let owner = Address::generate(&env);

        let factory_addr = env.register(old_stake::WASM, ());
        let old_stake_client = old_stake::Client::new(&env, &factory_addr);

        let lp_token_addr = env.register(
            TOKEN_WASM,
            (
                admin.clone(),
                7,
                String::from_str(&env, "LP Token"),
                String::from_str(&env, "LPT"),
            ),
        );

        let lp_token_client = token::Client::new(&env, &lp_token_addr);

        let reward_token_addr = env.register(
            TOKEN_WASM,
            (
                admin.clone(),
                7,
                String::from_str(&env, "Reward Token"),
                String::from_str(&env, "RWT"),
            ),
        );

        let reward_token_client = token::Client::new(&env, &reward_token_addr);
        reward_token_client.mint(&old_stake_client.address, &10_000_000_000_000);

        old_stake_client.initialize(
            &admin,
            &lp_token_client.address,
            &100,
            &50,
            &manager,
            &owner,
            &7,
        );

        old_stake_client.create_distribution_flow(&manager, &reward_token_addr);
    }
}
