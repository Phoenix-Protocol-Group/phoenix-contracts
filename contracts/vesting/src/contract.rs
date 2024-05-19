use soroban_sdk::{
    contract, contractimpl, contractmeta, log, panic_with_error, Address, BytesN, Env, Vec,
};

use crate::storage::{
    get_admin, get_token_info, save_max_vesting_complexity, save_token_info, DistributionInfo,
};
#[cfg(feature = "minter")]
use crate::storage::{get_minter, save_minter, MinterInfo};
use crate::{
    error::ContractError,
    storage::{
        get_max_vesting_complexity, get_vesting, save_admin, save_vesting, VestingInfo,
        VestingSchedule, VestingTokenInfo,
    },
    token_contract,
    utils::{check_duplications, validate_vesting_schedule},
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
        max_vesting_complexity: u32,
    );

    fn create_vesting_schedules(env: Env, vesting_accounts: Vec<VestingSchedule>);

    fn claim(env: Env, sender: Address);

    fn update(env: Env, new_wash_hash: BytesN<32>);

    fn query_balance(env: Env, address: Address) -> i128;

    fn query_distribution_info(env: Env, address: Address) -> DistributionInfo;

    fn query_token_info(env: Env) -> VestingTokenInfo;

    fn query_vesting_contract_balance(env: Env) -> i128;

    fn query_available_to_claim(env: Env, address: Address) -> i128;

    #[cfg(feature = "minter")]
    fn initialize_with_minter(
        env: Env,
        admin: Address,
        vesting_token: VestingTokenInfo,
        max_vesting_complexity: u32,
        minter_info: MinterInfo,
    );

    #[cfg(feature = "minter")]
    fn burn(env: Env, sender: Address, amount: u128);

    #[cfg(feature = "minter")]
    fn mint(env: Env, sender: Address, amount: i128);

    #[cfg(feature = "minter")]
    fn update_minter(env: Env, sender: Address, new_minter: Address);

    #[cfg(feature = "minter")]
    fn update_minter_capacity(env: Env, sender: Address, new_capacity: u128);

    #[cfg(feature = "minter")]
    fn query_minter(env: Env) -> MinterInfo;
}

#[contractimpl]
impl VestingTrait for Vesting {
    fn initialize(
        env: Env,
        admin: Address,
        vesting_token: VestingTokenInfo,
        max_vesting_complexity: u32,
    ) {
        save_admin(&env, &admin);

        let token_info = VestingTokenInfo {
            name: vesting_token.name,
            symbol: vesting_token.symbol,
            decimals: vesting_token.decimals,
            address: vesting_token.address,
        };

        save_token_info(&env, &token_info);
        save_max_vesting_complexity(&env, &max_vesting_complexity);

        env.events()
            .publish(("Initialize", "Vesting contract with admin: "), admin);
    }

    #[cfg(feature = "minter")]
    fn initialize_with_minter(
        env: Env,
        admin: Address,
        vesting_token: VestingTokenInfo,
        max_vesting_complexity: u32,
        minter_info: MinterInfo,
    ) {
        save_admin(&env, &admin);

        save_minter(&env, &minter_info);

        let token_info = VestingTokenInfo {
            name: vesting_token.name,
            symbol: vesting_token.symbol,
            decimals: vesting_token.decimals,
            address: vesting_token.address,
        };

        save_token_info(&env, &token_info);
        save_max_vesting_complexity(&env, &max_vesting_complexity);

        env.events()
            .publish(("Initialize", "Vesting contract with admin: "), admin);
    }

    fn create_vesting_schedules(env: Env, vesting_schedules: Vec<VestingSchedule>) {
        let admin = get_admin(&env);
        admin.require_auth();

        if vesting_schedules.is_empty() {
            log!(
                &env,
                "Vesting: Initialize: At least one vesting schedule must be provided."
            );
            panic_with_error!(env, ContractError::MissingBalance);
        }

        check_duplications(&env, vesting_schedules.clone());
        let max_vesting_complexity = get_max_vesting_complexity(&env);

        let mut total_vested_amount = 0;

        vesting_schedules.into_iter().for_each(|vb| {
            validate_vesting_schedule(
                &env,
                &vb.distribution_info.get_curve(),
                vb.distribution_info.amount,
            )
            .expect("Invalid curve and amount");

            if max_vesting_complexity <= vb.distribution_info.get_curve().size() {
                log!(
                    &env,
                    "Vesting: Create vesting account: Invalid curve complexity for {}",
                    vb.recipient
                );
                panic_with_error!(env, ContractError::VestingComplexityTooHigh);
            }

            save_vesting(
                &env,
                &vb.recipient,
                &VestingInfo {
                    balance: vb.distribution_info.amount,
                    distribution_info: vb.distribution_info.clone(),
                },
            );

            total_vested_amount += vb.distribution_info.amount;
        });

        // check if the admin has enough tokens to start the vesting contract
        let vesting_token = get_token_info(&env);
        let token_client = token_contract::Client::new(&env, &vesting_token.address);

        if token_client.balance(&admin) < total_vested_amount as i128 {
            log!(
                &env,
                "Vesting: Initialize: Admin does not have enough tokens to start the vesting schedule"
            );
            panic_with_error!(env, ContractError::NoEnoughtTokensToStart);
        }

        token_client.transfer(
            &admin,
            &env.current_contract_address(),
            &(total_vested_amount as i128),
        );
    }

    fn claim(env: Env, sender: Address) {
        sender.require_auth();

        let available_to_claim = Self::query_available_to_claim(env.clone(), sender.clone());

        if available_to_claim <= 0 {
            log!(&env, "Vesting: Claim: No tokens available to claim");
            panic_with_error!(env, ContractError::NeverFullyVested);
        }

        let token_client = token_contract::Client::new(&env, &get_token_info(&env).address);

        let vesting_info = get_vesting(&env, &sender);
        let vested = vesting_info
            .distribution_info
            .get_curve()
            .value(env.ledger().timestamp());

        let sender_balance = vesting_info.balance;
        let sender_liquid = sender_balance // this checks if we can withdraw any vesting
            .checked_sub(vested)
            .unwrap_or_else(|| panic_with_error!(env, ContractError::NotEnoughBalance));

        if sender_liquid < available_to_claim as u128 {
            log!(
            &env,
            "Vesting: Verify Vesting Update Balances: Remaining amount must be at least equal to vested amount"
        );
            panic_with_error!(env, ContractError::CantMoveVestingTokens);
        }

        save_vesting(
            &env,
            &sender,
            &VestingInfo {
                balance: sender_balance - available_to_claim as u128,
                distribution_info: vesting_info.distribution_info,
            },
        );

        token_client.transfer(
            &env.current_contract_address(),
            &sender,
            &(available_to_claim),
        );

        env.events()
            .publish(("Claim", "Claimed tokens: "), available_to_claim);
    }

    #[cfg(feature = "minter")]
    fn burn(env: Env, sender: Address, amount: u128) {
        sender.require_auth();

        if amount == 0 {
            log!(&env, "Vesting: Burn: Invalid burn amount");
            panic_with_error!(env, ContractError::InvalidBurnAmount);
        }

        let token_client = token_contract::Client::new(&env, &get_token_info(&env).address);

        token_client.burn(&sender, &(amount as i128));

        env.events().publish(("Burn", "Burned from: "), sender);
        env.events().publish(("Burn", "Burned tokens: "), amount);
    }

    #[cfg(feature = "minter")]
    fn mint(env: Env, sender: Address, amount: i128) {
        sender.require_auth();

        if amount <= 0 {
            log!(&env, "Vesting: Mint: Invalid mint amount");
            panic_with_error!(env, ContractError::InvalidMintAmount);
        }

        // check if minter is set
        let minter = if let Some(minter) = get_minter(&env) {
            minter
        } else {
            log!(&env, "Vesting: Mint: Minter not found");
            panic_with_error!(env, ContractError::MinterNotFound);
        };

        // check if sender is minter
        if sender != minter.address {
            log!(&env, "Vesting: Mint: Not authorized to mint");
            panic_with_error!(env, ContractError::NotAuthorized);
        }

        // check if minter has enough to mint
        let minter_remainder = get_minter(&env)
            .map_or(0, |m| m.mint_capacity)
            .checked_sub(amount as u128)
            .unwrap_or_else(|| {
                log!(
                    &env,
                    "Vesting: Mint: Minter does not have enough capacity to mint"
                );
                panic_with_error!(env, ContractError::NotEnoughCapacity);
            });

        // mint to recipient
        let token_client = token_contract::Client::new(&env, &get_token_info(&env).address);
        token_client.mint(&env.current_contract_address(), &amount);

        // we update the minter
        save_minter(
            &env,
            &MinterInfo {
                address: minter.address,
                mint_capacity: minter_remainder,
            },
        );

        env.events().publish(("Mint", "sender: "), sender);
        env.events().publish(("Mint", "Minted tokens: "), amount);
    }

    #[cfg(feature = "minter")]
    fn update_minter(env: Env, sender: Address, new_minter: Address) {
        let current_minter = get_minter(&env);

        let is_authorized = if let Some(current_minter) = current_minter.clone() {
            sender == current_minter.address
        } else {
            sender == get_admin(&env)
        };

        if !is_authorized {
            log!(
                env,
                "Vesting: Update minter: Not authorized to update minter"
            );
            panic_with_error!(env, ContractError::NotAuthorized);
        }

        let mint_capacity = current_minter.map_or(0, |m| m.mint_capacity);
        save_minter(
            &env,
            &MinterInfo {
                address: new_minter.clone(),
                mint_capacity,
            },
        );

        env.events()
            .publish(("Update minter", "Updated minter to: "), new_minter);
    }

    #[cfg(feature = "minter")]
    fn update_minter_capacity(env: Env, sender: Address, new_capacity: u128) {
        if sender != get_admin(&env) {
            log!(
                &env,
                "Vesting: Update minter capacity: Only contract's admin can update the minter's capacity"
            );
            panic_with_error!(env, ContractError::NotAuthorized);
        }

        if let Some(minter) = get_minter(&env) {
            save_minter(
                &env,
                &MinterInfo {
                    address: minter.address,
                    mint_capacity: new_capacity,
                },
            );
        } else {
            log!(&env, "Vesting: Update Minter Capacity: Minter not found");
            panic_with_error!(env, ContractError::MinterNotFound);
        };

        env.events().publish(
            ("Update minter capacity", "Updated minter capacity to: "),
            new_capacity,
        );
    }

    fn query_balance(env: Env, address: Address) -> i128 {
        token_contract::Client::new(&env, &get_token_info(&env).address).balance(&address)
    }

    fn query_distribution_info(env: Env, address: Address) -> DistributionInfo {
        get_vesting(&env, &address).distribution_info
    }

    fn query_token_info(env: Env) -> VestingTokenInfo {
        get_token_info(&env)
    }

    #[cfg(feature = "minter")]
    fn query_minter(env: Env) -> MinterInfo {
        if let Some(minter) = get_minter(&env) {
            minter
        } else {
            log!(&env, "Vesting: Query Minter: Minter not found");
            panic_with_error!(env, ContractError::MinterNotFound);
        }
    }

    fn query_vesting_contract_balance(env: Env) -> i128 {
        let token_address = get_token_info(&env).address;
        token_contract::Client::new(&env, &token_address).balance(&env.current_contract_address())
    }

    fn query_available_to_claim(env: Env, address: Address) -> i128 {
        let vesting_info = get_vesting(&env, &address);
        let vested = vesting_info
            .distribution_info
            .get_curve()
            .value(env.ledger().timestamp());

        let sender_balance = vesting_info.balance;
        let sender_liquid = sender_balance
            .checked_sub(vested)
            .unwrap_or_else(|| panic_with_error!(env, ContractError::NotEnoughBalance));

        sender_liquid as i128
    }

    fn update(env: Env, new_wasm_hash: BytesN<32>) {
        let admin = get_admin(&env);
        admin.require_auth();

        env.deployer().update_current_contract_wasm(new_wasm_hash);
    }
}
