use soroban_sdk::{
    contract, contractimpl, contractmeta, log, panic_with_error, vec, Address, Env, Vec,
};

use curve::Curve;

use crate::utils::{create_vesting_accounts, update_vesting, verify_vesting_and_transfer_tokens};
use crate::{
    error::ContractError,
    storage::{
        get_config, get_minter, get_vesting, get_vesting_total_supply, save_admin, save_config,
        save_minter, update_vesting_total_supply, Config, MinterInfo, VestingBalance,
        VestingTokenInfo,
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
        minter_info: Option<MinterInfo>,
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

    // TODO: we will need these in the future, not needed for the most basic implementation right now
    // TODO: replace the tuple `owner_spender: (Address, Address)` with how it is in `send_to_contract_from`
    // fn increase_allowance(env: Env, owner_spender: (Address, Address), amount: i128);

    // fn decrease_allowance(env: Env, owner_spender: (Address, Address), amount: i128);

    // fn transfer_from(
    //     env: Env,
    //     owner_spender: (Address, Address),
    //     to: Address,
    //     amount: i128,
    // ) -> Result<(), ContractError>;

    // fn burn_from(
    //     env: Env,
    //     sender: Address,
    //     owner: Address,
    //     amount: i128,
    // ) -> Result<(), ContractError>;

    // fn send_to_contract_from(
    //     env: Env,
    //     sender: Address,
    //     owner: Address,
    //     contract: Address,
    //     amount: i128,
    // ) -> Result<(), ContractError>;

    fn update_minter(env: Env, sender: Address, new_minter: Address);

    fn send_tokens_to_contract(env: Env, sender: Address, contract: Address, amount: i128);

    fn add_to_whitelist(env: Env, sender: Address, to_add: Address);

    fn remove_from_whitelist(env: Env, sender: Address, to_remove: Address);

    fn query_config(env: Env) -> Result<Config, ContractError>;

    fn query_balance(env: Env, address: Address) -> i128;

    fn query_vesting(env: Env, address: Address) -> Result<Curve, ContractError>;

    fn query_vesting_allowlist(env: Env) -> Vec<Address>;

    fn query_token_info(env: Env) -> VestingTokenInfo;

    fn query_minter(env: Env) -> Address;

    fn query_vesting_total_supply(env: Env) -> i128;
}

#[contractimpl]
impl VestingTrait for Vesting {
    fn initialize(
        env: Env,
        admin: Address,
        vesting_token: VestingTokenInfo,
        vesting_balances: Vec<VestingBalance>,
        minter_info: Option<MinterInfo>,
        allowed_vesters: Option<Vec<Address>>,
        max_vesting_complexity: u32,
    ) -> Result<(), ContractError> {
        save_admin(&env, &admin);

        let whitelisted_accounts = match allowed_vesters {
            Some(whitelisted) => whitelisted,
            None => vec![&env, admin.clone()],
        };

        // TODO: this check might make no sense
        if vesting_balances.is_empty() {
            log!(
                &env,
                "Vesting: Initialize: At least one balance must be provided."
            );
            panic_with_error!(env, ContractError::MissingBalance);
        }

        let total_supply = create_vesting_accounts(&env, max_vesting_complexity, vesting_balances)?;
        if let Some(mi) = minter_info {
            let cap = mi.cap.value(env.ledger().timestamp()) as i128;
            if total_supply > cap {
                log!(&env, "Vesting: Initialize: total supply over the cap");
                panic_with_error!(env, ContractError::SupplyOverTheCap);
            }
            save_minter(&env, mi);
        }

        let token_info = VestingTokenInfo {
            name: vesting_token.name,
            symbol: vesting_token.symbol,
            decimals: vesting_token.decimals,
            address: vesting_token.address,
            total_supply,
        };

        let config = Config {
            admin: admin.clone(),
            whitelist: whitelisted_accounts,
            token_info,
            max_vesting_complexity,
        };
        save_config(&env, &config);

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
            panic_with_error!(env, ContractError::InvalidTransferAmount);
        }

        verify_vesting_and_transfer_tokens(&env, &from, &to, amount)?;

        env.events().publish(
            (
                "Transfer token",
                "Transfering tokens between accounts: {}, {}, {}",
            ),
            (from, to, amount),
        );

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

        let whitelist = get_config(&env).whitelist;
        if !whitelist.contains(from.clone()) {
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
        if curve.value(env.ledger().timestamp()) != 0 {
            update_vesting(&env, &to, amount, curve)?;
        }

        verify_vesting_and_transfer_tokens(&env, &from, &to, amount)?;

        env.events().publish(
            (
                "Transfer vesting",
                "Transfering vesting between accounts: {}, {}, {}",
            ),
            (from, to, amount),
        );

        Ok(())
    }

    fn burn(env: Env, sender: Address, amount: i128) -> Result<(), ContractError> {
        sender.require_auth();

        if amount <= 0 {
            log!(&env, "Vesting: Burn: Invalid burn amount");
            panic_with_error!(env, ContractError::InvalidBurnAmount);
        }

        match get_vesting_total_supply(&env) - amount < 0 {
            true => {
                log!(
                    &env,
                    "Vesting: Burn: Critical error - total supply cannot be negative"
                );
                panic_with_error!(env, ContractError::Std);
            }
            false => update_vesting_total_supply(&env, get_vesting_total_supply(&env) - amount),
        };

        let token_client = token_contract::Client::new(&env, &get_config(&env).token_info.address);
        token_client.burn(&sender, &amount);

        env.events().publish(("Burn", "Burned tokens: "), amount);

        Ok(())
    }

    fn mint(env: Env, sender: Address, to: Address, amount: i128) {
        sender.require_auth();

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
                    "Vesting: Mint: Critical error - total supply overflow"
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

        env.events().publish(("Mint", "Minted tokens: "), amount);
    }

    // TODO: we will need these in the future, not needed for the most basic implementation right now
    // fn increase_allowance(env: Env, owner_spender: (Address, Address), amount: i128) {
    //     owner_spender.0.require_auth();

    //     if amount <= 0 {
    //         log!(&env, "Vesting: Increase allowance: Invalid amount");
    //         panic_with_error!(env, ContractError::InvalidAllowanceAmount);
    //     }

    //     let allowance = get_allowances(&env, &owner_spender)
    //         .checked_add(amount)
    //         .unwrap_or_else(|| {
    //             log!(
    //                 &env,
    //                 "Vesting: Increase allowance: Critical error - allowance cannot be negative"
    //             );
    //             panic_with_error!(env, ContractError::Std);
    //         });

    //     save_allowances(&env, &owner_spender, allowance);

    //     env.events().publish(
    //         (
    //             "Increase allowance",
    //             "Increased allowance between accounts: {}, {}, {}",
    //         ),
    //         (owner_spender.0, owner_spender.1, amount),
    //     );
    // }

    // fn decrease_allowance(env: Env, owner_spender: (Address, Address), amount: i128) {
    //     owner_spender.0.require_auth();

    //     if amount <= 0 {
    //         log!(&env, "Vesting: Decrease allowance: Invalid amount");
    //         panic_with_error!(env, ContractError::InvalidAllowanceAmount);
    //     }

    //     let allowance = get_allowances(&env, &owner_spender)
    //         .checked_sub(amount)
    //         .unwrap_or_else(|| {
    //             log!(
    //                 &env,
    //                 "Vesting: Decrease allowance: Critical error - allowance cannot be negative"
    //             );
    //             panic_with_error!(env, ContractError::Std);
    //         });

    //     save_allowances(&env, &owner_spender, allowance);

    //     env.events().publish(
    //         (
    //             "Decrease allowance",
    //             "Decreased allowance between accounts: {}, {}, {}",
    //         ),
    //         (owner_spender.0, owner_spender.1, amount),
    //     );
    // }

    // fn transfer_from(
    //     env: Env,
    //     owner_spender: (Address, Address),
    //     to: Address,
    //     amount: i128,
    // ) -> Result<(), ContractError> {
    //     let owner = owner_spender.0.clone();
    //     let spender = owner_spender.1.clone();
    //     spender.require_auth();

    //     if amount <= 0 {
    //         log!(&env, "Vesting: Transfer from: Invalid transfer amount");
    //         panic_with_error!(env, ContractError::InvalidTransferAmount);
    //     }

    //     // todo deduct_allowances
    //     let allowance = get_allowances(&env, &owner_spender);
    //     if allowance < amount {
    //         log!(&env, "Vesting: Transfer from: Not enough allowance");
    //         panic_with_error!(env, ContractError::NotEnoughBalance);
    //     }
    //     let new_allowance = allowance.checked_sub(amount).unwrap_or_else(|| {
    //         log!(
    //             &env,
    //             "Vesting: Transfer from: Critical error - allowance cannot be negative"
    //         );
    //         panic_with_error!(env, ContractError::Std);
    //     });

    //     verify_vesting_and_transfer_tokens(&env, &owner, &to, amount)?;

    //     save_allowances(&env, &owner_spender, new_allowance);

    //     env.events().publish(
    //         (
    //             "Transfer from",
    //             "Transfering tokens between accounts: {}, {}, {}",
    //         ),
    //         (owner, to, amount),
    //     );

    //     Ok(())
    // }

    // fn burn_from(
    //     env: Env,
    //     sender: Address,
    //     owner: Address,
    //     amount: i128,
    // ) -> Result<(), ContractError> {
    //     sender.require_auth();

    //     if amount <= 0 {
    //         log!(&env, "Vesting: Burn from: Invalid burn amount");
    //         panic_with_error!(env, ContractError::InvalidBurnAmount);
    //     }

    //     let allowance = get_allowances(&env, &(owner.clone(), sender.clone()));
    //     if allowance < amount {
    //         log!(&env, "Vesting: Burn from: Not enough allowance");
    //         panic_with_error!(env, ContractError::NotEnoughBalance);
    //     }

    //     let new_allowance = allowance.checked_sub(amount).unwrap_or_else(|| {
    //         log!(
    //             &env,
    //             "Vesting: Burn from: Critical error - allowance cannot be negative"
    //         );
    //         panic_with_error!(env, ContractError::Std);
    //     });

    //     let total_supply = get_vesting_total_supply(&env)
    //         .checked_sub(amount)
    //         .unwrap_or_else(|| {
    //             log!(
    //                 &env,
    //                 "Vesting: Burn from: Critical error - total supply cannot be negative"
    //             );
    //             panic_with_error!(env, ContractError::Std);
    //         });

    //     update_vesting_total_supply(&env, total_supply);

    //     let token_client = token_contract::Client::new(&env, &get_config(&env).token_info.address);
    //     token_client.burn(&owner, &amount);

    //     save_allowances(&env, &(owner, sender), new_allowance);

    //     env.events()
    //         .publish(("Burn from", "Burned tokens: "), amount);

    //     Ok(())
    // }

    // fn send_to_contract_from(
    //     env: Env,
    //     sender: Address,
    //     owner: Address,
    //     contract: Address,
    //     amount: i128,
    // ) -> Result<(), ContractError> {
    //     sender.require_auth();
    //     if amount <= 0 {
    //         log!(&env, "Vesting: Send to contract from: Invalid amount");
    //         panic_with_error!(env, ContractError::InvalidTransferAmount);
    //     }
    //     //used to verify that the sender is authorized by the owner
    //     let _ = get_allowances(&env, &(owner.clone(), sender.clone()));

    //     let token_client = token_contract::Client::new(&env, &get_config(&env).token_info.address);
    //     token_client.transfer(&owner, &contract, &amount);

    //     env.events().publish(
    //         (
    //             "Send to contract from",
    //             "Sent tokens to contract from account: {}, {}, {}",
    //         ),
    //         (owner, contract, amount),
    //     );

    //     Ok(())
    // }

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
                address: new_minter.clone(),
                cap: get_minter(&env).cap,
            },
        );

        env.events()
            .publish(("Update minter", "Updated minter to: "), new_minter);
    }

    fn send_tokens_to_contract(env: Env, sender: Address, contract: Address, amount: i128) {
        if amount <= 0 {
            log!(&env, "Vesting: Send tokens to contract: Invalid amount");
            panic_with_error!(env, ContractError::InvalidTransferAmount);
        }

        let token_client = token_contract::Client::new(&env, &get_config(&env).token_info.address);
        token_client.transfer(&sender, &contract, &amount);

        env.events().publish(
            (
                "Send tokens to contract",
                "Sent tokens to contract from account: {}, {}",
            ),
            (sender, contract),
        );
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

        config.whitelist.push_back(to_add.clone());
        save_config(&env, &config);

        env.events()
            .publish(("Add to whitelist", "Added to whitelist: "), to_add);
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

        if let Some(index) = config.whitelist.first_index_of(to_remove.clone()) {
            config.whitelist.remove(index);
        }

        save_config(&env, &config);

        env.events().publish(
            ("Remove from whitelist", "Removed from whitelist: "),
            to_remove,
        );
    }

    fn query_config(env: Env) -> Result<Config, ContractError> {
        let config = get_config(&env);

        Ok(config)
    }

    fn query_balance(env: Env, address: Address) -> i128 {
        let token_client = token_contract::Client::new(&env, &get_config(&env).token_info.address);
        token_client.balance(&address)
    }

    fn query_vesting(env: Env, address: Address) -> Result<Curve, ContractError> {
        let curve = get_vesting(&env, &address)?.curve;
        Ok(curve)
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

    fn query_vesting_total_supply(env: Env) -> i128 {
        get_vesting_total_supply(&env)
    }
}
