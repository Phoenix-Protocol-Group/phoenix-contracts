use soroban_sdk::{contract, contractimpl, contractmeta, log, panic_with_error, Address, Env, Vec};

use curve::Curve;

use crate::{
    error::ContractError,
    storage::{
        get_config, get_vesting, remove_vesting, save_admin, save_config, save_minter,
        save_vesting, Config, MinterInfo, VestingBalance, VestingTokenInfo,
    },
    token_contract,
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
        minter_info: MinterInfo,
        allowed_vesters: Option<Vec<Address>>,
        max_vesting_complexity: u32,
    ) -> Result<(), ContractError>;

    fn transfer_token(
        env: Env,
        from: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), ContractError>;

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
        minter_info: MinterInfo,
        allowed_vesters: Option<Vec<Address>>,
        max_vesting_complexity: u32,
    ) -> Result<(), ContractError> {
        save_admin(&env, &admin);

        let whitelisted_accounts = match allowed_vesters {
            Some(whitelisted) => whitelisted,
            None => Vec::new(&env),
        };

        let token_info = VestingTokenInfo {
            name: vesting_token.name,
            symbol: vesting_token.symbol,
            decimals: vesting_token.decimals,
            address: vesting_token.address,
        };

        let config = Config {
            admin: admin.clone(),
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

        let total_supply = create_vesting_accounts(&env, max_vesting_complexity, vesting_balances)?;
        let cap_limit = minter_info.cap.value(env.ledger().timestamp()) as i128;
        if total_supply > cap_limit {
            log!(&env, "Vesting: Initialize: total supply over the cap");
            panic_with_error!(env, ContractError::SupplyOverTheCap);
        };

        save_minter(&env, &minter_info.address, &minter_info.cap);

        env.events()
            .publish(("Initialize", "Vesting contract with admin: "), admin);

        Ok(())
    }

    fn transfer_token(
        env: Env,
        from: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), ContractError> {
        from.require_auth();

        if amount <= 0 {
            log!(&env, "Vesting: Transfer token: Invalid transfer amount");
            panic_with_error!(env, ContractError::InvalidZeroAmount);
        }

        let vesting_amount = get_vesting(&env, &from).0; // FIXME - probably use a struct

        // if vesting is equal to zero we can remove it
        if vesting_amount == 0 {
            remove_vesting(&env, &from)
        }

        let vesting_token_address = get_config(&env).token_info.address;

        let vestint_token_client = token_contract::Client::new(&env, &vesting_token_address);

        let sender_balance = vestint_token_client.balance(&from);
        if let Some(remainder) = sender_balance.checked_sub(amount) {
            if vesting_amount > remainder {
                log!(
                    &env,
                    "Vesting: Transfer Token: Remaining amount must be at least equal to vested amount"
                );
                panic_with_error!(env, ContractError::CantMoveVestingTokens);
            }
            vestint_token_client.transfer(&from, &to, &amount)
        } else {
            log!(
                &env,
                "Vesting: Transfer Token: Not enough balance to transfer"
            );
            panic_with_error!(env, ContractError::NotEnoughBalance);
        }

        Ok(())
    }

    fn transfer_vesting(env: Env, from: Address, to: Address, amount: i128, curve: Curve) {
        // check if caller is in the allowlist if not throw exception

        // check if amount is not zero

        // assert vesting is valid

        // if not fully vested we update

        // transfer
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
