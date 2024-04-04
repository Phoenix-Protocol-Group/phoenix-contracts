use soroban_sdk::{contract, contractimpl, contractmeta, log, panic_with_error, Address, Env, Vec};

use curve::Curve;

use crate::{
    error::ContractError,
    storage::{
        get_allowances, get_config, get_delegated, get_minter, get_vesting,
        get_vesting_total_supply, remove_vesting, save_admin, save_config, save_minter,
        save_vesting, update_vesting, update_vesting_total_supply, Config, MinterInfo,
        VestingBalance, VestingInfo, VestingTokenInfo,
    },
    token_contract,
    utils::{deduct_coins, transfer},
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

    fn transfer_vesting(
        env: Env,
        from: Address,
        to: Address,
        amount: i128,
        curve: Curve,
    ) -> Result<(), ContractError>;

    fn burn(env: Env, sender: Address, amount: i128) -> Result<(), ContractError>;

    fn mint(env: Env, sender: Address, to: Address, amount: i128);

    fn update_minter(env: Env, sender: Address, new_minter: Address);

    fn send_tokens_to_contract(env: Env, sender: Address, contract: Address, amount: i128);

    fn add_to_whitelist(env: Env, sender: Address, to_add: Address);

    fn remove_from_whitelist(env: Env, sender: Address, to_remove: Address);

    fn query_config(env: Env) -> Result<Config, ContractError>;

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
            total_supply: 0,
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

        save_minter(&env, minter_info);

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
        from.require_auth(); // what if the caller is different from the sender?

        if amount <= 0 {
            log!(&env, "Vesting: Transfer token: Invalid transfer amount");
            panic_with_error!(env, ContractError::InvalidTransferAmount);
        }

        let vesting_amount_result = deduct_coins(&env, &from, amount)?;

        transfer(
            &env,
            &env.current_contract_address(),
            &to,
            amount,
            vesting_amount_result,
        )?;

        Ok(())
    }

    fn transfer_vesting(
        env: Env,
        from: Address,
        to: Address,
        amount: i128,
        curve: Curve,
    ) -> Result<(), ContractError> {
        from.require_auth();

        let white_list = get_config(&env).whitelist;
        if !white_list.contains(from.clone()) {
            log!(
                &env,
                "Vesting: Transfer Vesting: Not authorized to transfer vesting"
            );
            panic_with_error!(env, ContractError::NotAuthorized);
        }

        if amount <= 0 {
            log!(&env, "Vesting: Transfer Vesting: Invalid transfer amount");
            panic_with_error!(env, ContractError::InvalidTransferAmount);
        }

        curve.validate_monotonic_decreasing()?;
        let (low, high) = curve.range();
        if low != 0 {
            log!(&env, "Vesting: Transfer Vesting: Invalid low value");
            panic_with_error!(env, ContractError::NeverFullyVested);
        } else if high as i128 > amount {
            log!(
                &env,
                "Vesting: Transfer Vesting: Vesting more than being sent"
            );
            panic_with_error!(env, ContractError::VestsMoreThanSent);
        }

        // if not fully vested we update
        if !curve.value(env.ledger().timestamp()) == 0 {
            update_vesting(&env, &to, curve)?;
        }

        let vesting_amount_result = deduct_coins(&env, &from, amount)?;

        transfer(
            &env,
            &env.current_contract_address(),
            &to,
            amount,
            vesting_amount_result,
        )?;
        Ok(())
    }

    fn burn(env: Env, sender: Address, amount: i128) -> Result<(), ContractError> {
        // verity the amount
        if amount <= 0 {
            log!(&env, "Vesting: Burn: Invalid burn amount");
            panic_with_error!(env, ContractError::InvalidBurnAmount);
        }

        // deduct the amount from the sender
        let _ = deduct_coins(&env, &sender, amount)?;

        let total_supply = get_vesting_total_supply(&env)
            .checked_sub(amount)
            .unwrap_or_else(|| {
                log!(
                    &env,
                    "Vesting: Burn: Critical error - total supply cannot be negative"
                );
                panic_with_error!(env, ContractError::Std);
            });

        update_vesting_total_supply(&env, total_supply);

        Ok(())
    }

    fn mint(env: Env, sender: Address, to: Address, amount: i128) {
        // check amount
        if amount <= 0 {
            log!(&env, "Vesting: Mint: Invalid mint amount");
            panic_with_error!(env, ContractError::InvalidMintAmount);
        }

        // check if sender is minter
        if sender != get_minter(&env).address {
            log!(&env, "Vesting: Mint: Not authorized to mint");
            panic_with_error!(env, ContractError::NotAuthorized);
        }

        // update supply and cap
        let total_supply = get_vesting_total_supply(&env)
            .checked_add(amount)
            .unwrap_or_else(|| {
                log!(
                    &env,
                    "Vesting: Mint: Critical error - total supply cannot be negative"
                );
                panic_with_error!(env, ContractError::Std);
            });

        update_vesting_total_supply(&env, total_supply);

        let limit = get_minter(&env).cap.value(env.ledger().timestamp());
        if total_supply > limit as i128 {
            log!(&env, "Vesting: Mint: total supply over the cap");
            panic_with_error!(env, ContractError::SupplyOverTheCap);
        }

        // mint to recipient
        let token_client = token_contract::Client::new(&env, &get_config(&env).token_info.address);
        token_client.mint(&to, &amount);
    }

    fn update_minter(env: Env, sender: Address, new_minter: Address) {
        if sender != get_minter(&env).address {
            log!(
                &env,
                "Vesting: Update minter: Not authorized to update minter"
            );
            panic_with_error!(env, ContractError::NotAuthorized);
        }

        save_minter(
            &env,
            MinterInfo {
                address: new_minter,
                cap: get_minter(&env).cap,
            },
        );
    }

    fn send_tokens_to_contract(env: Env, sender: Address, contract: Address, amount: i128) {
        if amount <= 0 {
            log!(&env, "Vesting: Send tokens to contract: Invalid amount");
            panic_with_error!(env, ContractError::InvalidTransferAmount);
        }

        let _ = deduct_coins(&env, &sender, amount);
    }

    fn add_to_whitelist(env: Env, sender: Address, to_add: Address) {
        let mut config = get_config(&env);
        if sender != config.admin {
            log!(
                &env,
                "Vesting: Add to whitelist: Not authorized to add to whitelist"
            );
            panic_with_error!(env, ContractError::NotAuthorized);
        }

        config.whitelist.push_back(to_add);
        save_config(&env, &config);
    }

    fn remove_from_whitelist(env: Env, sender: Address, to_remove: Address) {
        let mut config = get_config(&env);
        if sender != config.admin {
            log!(
                &env,
                "Vesting: Remove from whitelist: Not authorized to remove from whitelist"
            );
            panic_with_error!(env, ContractError::NotAuthorized);
        }

        config.whitelist.first_index_of(to_remove).map(|index| {
            config.whitelist.remove(index);
        });

        save_config(&env, &config);
    }

    fn query_config(env: Env) -> Result<Config, ContractError> {
        let config = get_config(&env);

        Ok(config)
    }

    fn query_balance(env: Env, address: Address) -> i128 {
        let token_client = token_contract::Client::new(&env, &get_config(&env).token_info.address);
        token_client.balance(&address)
    }

    fn query_vesting(env: Env, address: Address) -> Curve {
        get_vesting(&env, &address).curve
    }

    fn query_delegated(env: Env, address: Address) -> i128 {
        get_delegated(&env, &address)
    }

    fn query_vesting_allowlist(env: Env) -> Vec<Address> {
        get_config(&env).whitelist
    }

    fn query_token_info(env: Env) -> VestingTokenInfo {
        get_config(&env).token_info
    }

    fn query_minter(env: Env) -> Address {
        get_minter(&env).address
    }

    fn query_allowance(env: Env, owner_spender: (Address, Address)) -> i128 {
        get_allowances(&env, &owner_spender)
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

        save_vesting(
            &env,
            &vb.address,
            VestingInfo {
                amount: vb.balance,
                curve: vb.curve,
            },
        );
        total_supply += vb.balance;
    });

    Ok(total_supply)
}
