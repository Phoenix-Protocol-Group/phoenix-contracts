use soroban_sdk::{contract, contractimpl, contractmeta, log, panic_with_error, Address, Env, Vec};

use curve::Curve;

use crate::storage::{get_admin, get_token_info, save_max_vesting_complexity, save_token_info};
use crate::utils::{create_vesting_accounts, verify_vesting};
use crate::{
    error::ContractError,
    storage::{
        get_minter, get_vesting, get_vesting_total_supply, save_admin, save_minter,
        update_vesting_total_supply, MinterInfo, VestingBalance, VestingTokenInfo,
    },
    token_contract,
};

// Metadata that is added on to the WASM custom section
contractmeta!(
    key = "Description",
    val = "Phoenix Protocol Token Vesting Contract"
);
#[contract]
pub struct Vesting;

pub trait VestingTrait {
    fn initialize(
        env: Env,
        admin: Address,
        vesting_token: VestingTokenInfo,
        vesting_balances: Vec<VestingBalance>,
        minter_info: Option<MinterInfo>,
        max_vesting_complexity: u32,
    ) -> Result<(), ContractError>;

    fn transfer_token(
        env: Env,
        from: Address,
        to: Address,
        amount: i128,
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

    fn update_minter_capacity(
        env: Env,
        sender: Address,
        new_capacity: Curve,
        remove_old_capacity: bool,
    );

    fn query_balance(env: Env, address: Address) -> i128;

    fn query_vesting(env: Env, address: Address) -> Result<Curve, ContractError>;

    fn query_token_info(env: Env) -> VestingTokenInfo;

    fn query_minter(env: Env) -> MinterInfo;

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
        max_vesting_complexity: u32,
    ) -> Result<(), ContractError> {
        save_admin(&env, &admin);

        if vesting_balances.is_empty() {
            log!(
                &env,
                "Vesting: Initialize: At least one vesting schedule must be provided."
            );
            panic_with_error!(env, ContractError::MissingBalance);
        }

        let total_supply = create_vesting_accounts(&env, max_vesting_complexity, vesting_balances)?;
        if let Some(mi) = minter_info {
            let capacity = mi.capacity.value(env.ledger().timestamp()) as i128;
            if total_supply > capacity {
                log!(&env, "Vesting: Initialize: total supply over the capacity");
                panic_with_error!(env, ContractError::SupplyOverTheCap);
            }
            save_minter(&env, &mi);
        }

        let token_info = VestingTokenInfo {
            name: vesting_token.name,
            symbol: vesting_token.symbol,
            decimals: vesting_token.decimals,
            address: vesting_token.address,
            total_supply,
        };

        save_token_info(&env, &token_info);
        save_max_vesting_complexity(&env, &max_vesting_complexity);

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

        let token_client = token_contract::Client::new(&env, &get_token_info(&env).address);

        verify_vesting(&env, &from, amount, &token_client)?;
        token_client.transfer(&from, &to, &amount);

        env.events().publish(
            (
                "Transfer token",
                "Transfering tokens between accounts: from: {}, to:{}, amount: {}",
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

        let remainder = get_vesting_total_supply(&env) - amount;
        if remainder >= 0 {
            update_vesting_total_supply(&env, remainder);
        } else {
            log!(
                &env,
                "Vesting: Burn: Critical error - total supply cannot be negative"
            );
            panic_with_error!(env, ContractError::Std);
        };
        let token_client = token_contract::Client::new(&env, &get_token_info(&env).address);

        verify_vesting(&env, &sender, amount, &token_client)?;
        token_client.burn(&sender, &amount);

        env.events().publish(("Burn", "Burned from: "), sender);
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

        // update supply and capacity
        let updated_total_supply = get_vesting_total_supply(&env)
            .checked_add(amount)
            .unwrap_or_else(|| {
                log!(
                    &env,
                    "Vesting: Mint: Critical error - total supply overflow"
                );
                panic_with_error!(env, ContractError::Std);
            });

        update_vesting_total_supply(&env, updated_total_supply);

        let limit = get_minter(&env).capacity.value(env.ledger().timestamp());
        if updated_total_supply >= limit as i128 {
            log!(&env, "Vesting: Mint: total supply over the capacity");
            panic_with_error!(env, ContractError::SupplyOverTheCap);
        }

        // mint to recipient
        let token_client = token_contract::Client::new(&env, &get_token_info(&env).address);
        token_client.mint(&to, &amount);

        env.events().publish(("Mint", "sender: "), sender);
        env.events().publish(("Mint", "Recipient: "), to);
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
    //             "Increased allowance between accounts: from: {}, to: {}, increase: {}",
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
    //             "Decreased allowance between accounts: from: {}, to: {}, decrease: {}",
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
    //             "Transfering tokens between accounts: from: {}, to: {}, amount: {}",
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
    //             "Sent tokens to contract from account: from: {}, to: {}, amount: {}",
    //         ),
    //         (owner, contract, amount),
    //     );

    //     Ok(())
    // }

    fn update_minter(env: Env, sender: Address, new_minter: Address) {
        if sender != get_minter(&env).address && sender != get_admin(&env) {
            log!(
                &env,
                "Vesting: Update minter: Not authorized to update minter"
            );
            panic_with_error!(env, ContractError::NotAuthorized);
        }

        save_minter(
            &env,
            &MinterInfo {
                address: new_minter.clone(),
                capacity: get_minter(&env).capacity,
            },
        );

        env.events()
            .publish(("Update minter", "Updated minter to: "), new_minter);
    }

    fn update_minter_capacity(
        env: Env,
        sender: Address,
        new_capacity: Curve,
        remove_old_capacity: bool,
    ) {
        if sender != get_admin(&env) {
            log!(
                &env,
                "Vesting: Update minter capacity: Only contract's admin can update the minter's capacity"
            );
            panic_with_error!(env, ContractError::NotAuthorized);
        }

        match remove_old_capacity {
            true => {
                save_minter(
                    &env,
                    &MinterInfo {
                        address: get_minter(&env).address,
                        capacity: new_capacity.clone(),
                    },
                );
            }
            false => {
                // TODO: we will eventually need to verify the new minter capacity curve complexity at some point
                let new_curve_capacity = get_minter(&env).capacity.combine(&env, &new_capacity);
                save_minter(
                    &env,
                    &MinterInfo {
                        address: get_minter(&env).address,
                        capacity: new_curve_capacity,
                    },
                );
            }
        }

        env.events().publish(
            ("Update minter capacity", "Updated minter capacity to: "),
            new_capacity,
        );
    }

    fn query_balance(env: Env, address: Address) -> i128 {
        token_contract::Client::new(&env, &get_token_info(&env).address).balance(&address)
    }

    fn query_vesting(env: Env, address: Address) -> Result<Curve, ContractError> {
        Ok(get_vesting(&env, &address)?.curve)
    }

    fn query_token_info(env: Env) -> VestingTokenInfo {
        get_token_info(&env)
    }

    fn query_minter(env: Env) -> MinterInfo {
        get_minter(&env)
    }

    fn query_vesting_total_supply(env: Env) -> i128 {
        get_vesting_total_supply(&env)
    }
}
