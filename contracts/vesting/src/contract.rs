use soroban_sdk::{contract, contractimpl, contractmeta, log, panic_with_error, Address, Env, Vec};

use curve::Curve;

use crate::{
    error::ContractError,
    storage::{
        save_admin, save_balance, save_config, save_minter, save_vesting, save_vesting_schedule,
        Config, VestingBalance, VestingTokenInfo,
    },
};

// Metadata that is added on to the WASM custom section
contractmeta!(key = "Description", val = "Phoenix Protocol Vesting");

#[contract]
pub struct Vesting;

pub trait VestingTrait {
    fn initialize(
        env: Env,
        admin: Address,
        vesting_token: VestingTokenInfo,
        vesting_balances: Vec<VestingBalance>,
        minter_addr: Address,
        allowed_vesters: Option<Vec<Address>>,
        max_vesting_complexity: u32,
    );

    fn create_vesting_accounts(env: Env, accounts: Vec<Address>);

    fn transfer_token(env: Env, from: Address, to: Address, amount: i128);

    fn transfer_vesting(env: Env, from: Address, to: Address, amount: i128, curve: Curve);

    fn burn(env: Env, amount: i128);

    fn mint(env: Env, sender: Address, to: Address, amount: i128);

    fn update_minter(env: Env, sender: Address, new_minter: Address);

    fn send_tokens_to_contract(env: Env, sender: Address, contract: Address, amount: i128);

    fn add_to_whitelist(env: Env, sender: Address, to_add: Address);

    fn remove_from_whitelist(env: Env, sender: Address, to_remove: Address);

    fn query_config(env: Env) -> Config;

    fn query_balance(env: Env, address: Address) -> i128;

    fn query_vesting(env: Env, address: Address) -> Curve;

    fn query_delegated(env: Env, address: Address) -> i128;

    fn query_vesting_allowlist(env: Env) -> Vec<Address>;

    fn query_token_info(env: Env) -> VestingTokenInfo;

    fn query_minter(env: Env) -> Address;

    fn query_allowance(env: Env, owner_spender: (Address, Address)) -> i128;
}

#[contractimpl]
impl VestingTrait for Vesting {
    fn initialize(
        env: Env,
        admin: Address,
        vesting_token: VestingTokenInfo,
        vesting_balances: Vec<VestingBalance>,
        minter_addr: Address,
        allowed_vesters: Option<Vec<Address>>,
        max_vesting_complexity: u32,
    ) {
        save_admin(&env, &admin);

        let whitelisted_accounts = match allowed_vesters {
            Some(whitelisted) => whitelisted,
            None => Vec::new(&env),
        };

        let token_info = VestingTokenInfo {
            name: vesting_token.name,
            symbol: vesting_token.symbol,
            decimals: vesting_token.decimals,
        };

        let config = Config {
            admin,
            whitelist: whitelisted_accounts,
            token_info,
            max_vesting_complexity,
        };

        save_config(&env, &config);

        if vesting_balances.len() <= 0 {
            log!(
                &env,
                "Vesting: Initialize: At least one balance must be provided."
            );
            panic_with_error!(env, ContractError::MissingBalance);
        }

        create_vesting_accounts(&env, max_vesting_complexity, vesting_balances);

        save_minter(&env, &minter_addr);
    }

    fn create_vesting_accounts(env: Env, accounts: Vec<Address>) {
        todo!("create_vesting_accounts")
    }

    fn transfer_token(env: Env, from: Address, to: Address, amount: i128) {
        todo!("transfer_token")
    }

    fn transfer_vesting(env: Env, from: Address, to: Address, amount: i128, curve: Curve) {
        todo!("transfer_vesting")
    }

    fn burn(env: Env, amount: i128) {
        todo!("burn")
    }

    fn mint(env: Env, sender: Address, to: Address, amount: i128) {
        todo!("mint")
    }

    fn update_minter(env: Env, sender: Address, new_minter: Address) {
        todo!("update_minter")
    }

    fn send_tokens_to_contract(env: Env, sender: Address, contract: Address, amount: i128) {
        todo!("send_tokens_to_contract")
    }

    fn add_to_whitelist(env: Env, sender: Address, to_add: Address) {
        todo!("add_to_whitelist")
    }

    fn remove_from_whitelist(env: Env, sender: Address, to_remove: Address) {
        todo!("remove_from_whitelist")
    }

    fn query_config(env: Env) -> Config {
        todo!("query_config")
    }

    fn query_balance(env: Env, address: Address) -> i128 {
        todo!("query_balance")
    }

    fn query_vesting(env: Env, address: Address) -> Curve {
        todo!("query_vesting")
    }

    fn query_delegated(env: Env, address: Address) -> i128 {
        todo!("query_delegated")
    }

    fn query_vesting_allowlist(env: Env) -> Vec<Address> {
        todo!("query_vesting_allowlist")
    }

    fn query_token_info(env: Env) -> VestingTokenInfo {
        todo!("query_token_info")
    }

    fn query_minter(env: Env) -> Address {
        todo!("query_minter")
    }

    fn query_allowance(env: Env, owner_spender: (Address, Address)) -> i128 {
        todo!("query_allowance")
    }
}

fn create_vesting_accounts(
    env: &Env,
    vesting_complexity: u32,
    vesting_balances: Vec<VestingBalance>,
) -> Result<i128, ContractError> {
    let mut total_supply = 0;

    vesting_balances.into_iter().for_each(|vb| {
        if vesting_complexity <= vb.curve.size() {
            log!(
                &env,
                "Vesting: Create vesting account: Invalid curve complexity for {}",
                vb.address
            );
            panic_with_error!(env, ContractError::VestingComplexityTooHigh);
        }

        save_vesting(&env, &vb.address, (vb.balance, vb.curve));
        total_supply += vb.balance;
    });

    Ok(total_supply)
}
