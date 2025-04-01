use phoenix::{
    ttl::{INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL},
    utils::{convert_i128_to_u128, convert_u128_to_i128, AdminChange},
};
use soroban_sdk::{
    contract, contractimpl, contractmeta, log, panic_with_error, vec, Address, BytesN, Env, String,
    Vec,
};

#[cfg(feature = "minter")]
use crate::storage::{get_minter, save_minter, MinterInfo};
use crate::{
    error::ContractError,
    storage::{
        get_admin_old, get_max_vesting_complexity, get_token_info, get_vesting, is_initialized,
        save_admin_old, save_max_vesting_complexity, save_token_info, save_vesting,
        set_initialized, update_vesting, VestingCounterKey, VestingInfo, VestingInfoKey,
        VestingInfoResponse, VestingSchedule, VestingTokenInfo, ADMIN, PENDING_ADMIN, VESTING_KEY,
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

#[allow(dead_code)]
pub trait VestingTrait {
    fn initialize(
        env: Env,
        admin: Address,
        vesting_token: VestingTokenInfo,
        max_vesting_complexity: u32,
    );

    fn create_vesting_schedules(env: Env, vesting_accounts: Vec<VestingSchedule>);

    fn claim(env: Env, sender: Address, index: u64);

    fn update(env: Env, new_wash_hash: BytesN<32>);

    fn query_balance(env: Env, address: Address) -> i128;

    fn query_vesting_info(env: Env, address: Address, index: u64) -> VestingInfoResponse;

    fn query_all_vesting_info(env: Env, address: Address) -> Vec<VestingInfoResponse>;

    fn query_token_info(env: Env) -> VestingTokenInfo;

    fn query_vesting_contract_balance(env: Env) -> i128;

    fn query_available_to_claim(env: Env, address: Address, index: u64) -> i128;

    fn update_vesting_token(env: Env, new_token_address: Address) -> Result<(), ContractError>;

    fn update_max_complexity(env: Env, new_max_complexity: u32) -> Result<(), ContractError>;

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

    fn migrate_admin_key(env: Env) -> Result<(), ContractError>;

    fn propose_admin(
        env: Env,
        new_admin: Address,
        time_limit: Option<u64>,
    ) -> Result<Address, ContractError>;

    fn revoke_admin_change(env: Env) -> Result<(), ContractError>;

    fn accept_admin(env: Env) -> Result<Address, ContractError>;
}

#[contractimpl]
impl VestingTrait for Vesting {
    fn initialize(
        env: Env,
        admin: Address,
        vesting_token: VestingTokenInfo,
        max_vesting_complexity: u32,
    ) {
        if is_initialized(&env) {
            log!(
                &env,
                "Stake: Initialize: initializing contract twice is not allowed"
            );
            panic_with_error!(&env, ContractError::AlreadyInitialized);
        }

        set_initialized(&env);

        save_admin_old(&env, &admin);

        let token_info = VestingTokenInfo {
            name: vesting_token.name,
            symbol: vesting_token.symbol,
            decimals: vesting_token.decimals,
            address: vesting_token.address,
        };

        save_token_info(&env, &token_info);
        save_max_vesting_complexity(&env, &max_vesting_complexity);

        env.storage().persistent().set(&VESTING_KEY, &true);

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
        if is_initialized(&env) {
            log!(
                &env,
                "Stake: Initialize: initializing contract twice is not allowed"
            );
            panic_with_error!(&env, ContractError::AlreadyInitialized);
        }

        set_initialized(&env);
        save_admin_old(&env, &admin);

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
        let admin = get_admin_old(&env);
        admin.require_auth();
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        if vesting_schedules.is_empty() {
            log!(
                &env,
                "Vesting: Create vesting account: At least one vesting schedule must be provided."
            );
            panic_with_error!(env, ContractError::MissingBalance);
        }

        check_duplications(&env, vesting_schedules.clone());
        let max_vesting_complexity = get_max_vesting_complexity(&env);

        let mut total_vested_amount: u128 = 0;

        vesting_schedules.into_iter().for_each(|vesting_schedule| {
            let vested_amount = validate_vesting_schedule(&env, &vesting_schedule.curve)
                .expect("Invalid curve and amount");

            if max_vesting_complexity <= vesting_schedule.curve.size() {
                log!(
                    &env,
                    "Vesting: Create vesting account: Invalid curve complexity for {}",
                    vesting_schedule.recipient
                );
                panic_with_error!(env, ContractError::VestingComplexityTooHigh);
            }

            save_vesting(
                &env,
                &vesting_schedule.recipient.clone(),
                VestingInfo {
                    balance: vested_amount,
                    recipient: vesting_schedule.recipient,
                    schedule: vesting_schedule.curve.clone(),
                },
            );

            total_vested_amount = total_vested_amount
                .checked_add(vested_amount)
                .unwrap_or_else(|| {
                    log!(&env, "Vesting: Create Vesting Schedule: overflow ocurred.");
                    panic_with_error!(&env, ContractError::ContractMathError);
                });
        });

        // check if the admin has enough tokens to start the vesting contract
        let vesting_token = get_token_info(&env);
        let token_client = token_contract::Client::new(&env, &vesting_token.address);

        if token_client.balance(&admin) < convert_u128_to_i128(total_vested_amount) {
            log!(
                &env,
                "Vesting: Create vesting account: Admin does not have enough tokens to start the vesting schedule"
            );
            panic_with_error!(env, ContractError::NoEnoughtTokensToStart);
        }

        token_client.transfer(
            &admin,
            &env.current_contract_address(),
            &convert_u128_to_i128(total_vested_amount),
        );
    }

    fn claim(env: Env, sender: Address, index: u64) {
        sender.require_auth();
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        let available_to_claim = Self::query_available_to_claim(env.clone(), sender.clone(), index);

        if available_to_claim <= 0 {
            log!(&env, "Vesting: Claim: No tokens available to claim");
            panic_with_error!(env, ContractError::NeverFullyVested);
        }

        let token_client = token_contract::Client::new(&env, &get_token_info(&env).address);

        let vesting_info = get_vesting(&env, &sender, index);
        let vested = vesting_info.schedule.value(env.ledger().timestamp());

        let sender_balance = vesting_info.balance;
        let sender_liquid = sender_balance // this checks if we can withdraw any vesting
            .checked_sub(vested)
            .unwrap_or_else(|| panic_with_error!(env, ContractError::NotEnoughBalance));

        if sender_liquid < convert_i128_to_u128(available_to_claim) {
            log!(
            &env,
            "Vesting: Verify Vesting Update Balances: Remaining amount must be at least equal to vested amount"
        );
            panic_with_error!(env, ContractError::CantMoveVestingTokens);
        }

        let updated_balance = sender_balance
            .checked_sub(convert_i128_to_u128(available_to_claim))
            .unwrap_or_else(|| {
                log!(&env, "Vesting: Claim: underflow occured");
                panic_with_error!(&env, ContractError::ContractMathError);
            });
        update_vesting(
            &env,
            &sender,
            index,
            &VestingInfo {
                balance: updated_balance,
                ..vesting_info
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
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

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
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

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
            .checked_sub(convert_i128_to_u128(amount))
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
        sender.require_auth();
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        let current_minter = get_minter(&env);

        let is_authorized = if let Some(current_minter) = current_minter.clone() {
            sender == current_minter.address
        } else {
            sender == get_admin_old(&env)
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
        sender.require_auth();
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        if sender != get_admin_old(&env) {
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
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);
        token_contract::Client::new(&env, &get_token_info(&env).address).balance(&address)
    }

    fn query_vesting_info(env: Env, address: Address, index: u64) -> VestingInfoResponse {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);
        let vesting_info = get_vesting(&env, &address, index);
        VestingInfoResponse {
            balance: vesting_info.balance,
            recipient: vesting_info.recipient,
            schedule: vesting_info.schedule,
            index, // use the query parameter index
        }
    }

    fn query_all_vesting_info(env: Env, address: Address) -> Vec<VestingInfoResponse> {
        let counter_key = VestingCounterKey {
            recipient: address.clone(),
        };
        let count: u64 = env.storage().persistent().get(&counter_key).unwrap_or(0);
        let mut responses = vec![&env];

        for i in 0..count {
            if env
                .storage()
                .persistent()
                .get::<VestingInfoKey, VestingInfo>(&VestingInfoKey {
                    recipient: address.clone(),
                    index: i,
                })
                .is_some()
            {
                let vesting = get_vesting(&env, &address, i);
                responses.push_back(VestingInfoResponse {
                    balance: vesting.balance,
                    recipient: vesting.recipient,
                    schedule: vesting.schedule,
                    index: i,
                });
            }
        }
        responses
    }

    fn query_token_info(env: Env) -> VestingTokenInfo {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);
        get_token_info(&env)
    }

    #[cfg(feature = "minter")]
    fn query_minter(env: Env) -> MinterInfo {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);
        if let Some(minter) = get_minter(&env) {
            minter
        } else {
            log!(&env, "Vesting: Query Minter: Minter not found");
            panic_with_error!(env, ContractError::MinterNotFound);
        }
    }

    fn query_vesting_contract_balance(env: Env) -> i128 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);
        let token_address = get_token_info(&env).address;
        token_contract::Client::new(&env, &token_address).balance(&env.current_contract_address())
    }

    fn query_available_to_claim(env: Env, address: Address, index: u64) -> i128 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);
        let vesting_info = get_vesting(&env, &address, index);

        let difference = vesting_info
            .balance
            .checked_sub(vesting_info.schedule.value(env.ledger().timestamp()))
            .unwrap_or_else(|| {
                log!(
                    &env,
                    "Vesting: Query Available To Claim: underflow occured."
                );
                panic_with_error!(&env, ContractError::ContractMathError);
            });
        convert_u128_to_i128(difference)
    }
    fn update_vesting_token(env: Env, new_token_address: Address) -> Result<(), ContractError> {
        get_admin_old(&env).require_auth();

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        let old_token_info = get_token_info(&env);
        if old_token_info.address == new_token_address {
            log!(&env, "Vesting: Update Token: New address is same as old.");
            panic_with_error!(&env, ContractError::SameTokenAddress);
        }

        let new_token_client = token_contract::Client::new(&env, &new_token_address);
        let new_name = new_token_client.name();
        let new_symbol = new_token_client.symbol();
        let new_decimals = new_token_client.decimals();

        let new_token_info = VestingTokenInfo {
            name: new_name.clone(),
            symbol: new_symbol.clone(),
            decimals: new_decimals,
            address: new_token_address.clone(),
        };

        save_token_info(&env, &new_token_info);

        env.events()
            .publish(("Vesting: Update Token", "new_addr:"), new_token_address);

        env.events().publish(
            (
                "Vesting: Update Token",
                "new_name:",
                "new_symbol:",
                "new_decimals:",
            ),
            (new_name, new_symbol, new_decimals),
        );

        Ok(())
    }

    fn update_max_complexity(env: Env, new_max_complexity: u32) -> Result<(), ContractError> {
        get_admin_old(&env).require_auth();

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        if new_max_complexity == 0 {
            log!(
                &env,
                "Vesting: Update Max Complexity: Complexity cannot be zero."
            );
            panic_with_error!(&env, ContractError::InvalidMaxComplexity);
        }

        save_max_vesting_complexity(&env, &new_max_complexity);

        env.events().publish(
            ("Vesting: Update Max Complexity", "new:"),
            new_max_complexity,
        );

        Ok(())
    }

    fn update(env: Env, new_wasm_hash: BytesN<32>) {
        let admin = get_admin_old(&env);
        admin.require_auth();

        env.deployer().update_current_contract_wasm(new_wasm_hash);
    }

    fn migrate_admin_key(env: Env) -> Result<(), ContractError> {
        let admin = get_admin_old(&env);
        env.storage().instance().set(&ADMIN, &admin);

        Ok(())
    }

    fn propose_admin(
        env: Env,
        new_admin: Address,
        time_limit: Option<u64>,
    ) -> Result<Address, ContractError> {
        let current_admin = get_admin_old(&env);
        current_admin.require_auth();

        if current_admin == new_admin {
            log!(&env, "Trying to set new admin as new");
            panic_with_error!(&env, ContractError::SameAdmin);
        }

        env.storage().instance().set(
            &PENDING_ADMIN,
            &AdminChange {
                new_admin: new_admin.clone(),
                time_limit,
            },
        );

        env.events().publish(
            ("Vesting: ", "Admin replacement requested by old admin: "),
            &current_admin,
        );
        env.events()
            .publish(("Vesting: ", "Replace with new admin: "), &new_admin);

        Ok(new_admin)
    }

    fn revoke_admin_change(env: Env) -> Result<(), ContractError> {
        let current_admin = get_admin_old(&env);
        current_admin.require_auth();

        if !env.storage().instance().has(&PENDING_ADMIN) {
            log!(&env, "No admin change in place");
            panic_with_error!(&env, ContractError::NoAdminChangeInPlace);
        }

        env.storage().instance().remove(&PENDING_ADMIN);

        env.events()
            .publish(("Trader: ", "Undo admin change: "), ());

        Ok(())
    }

    fn accept_admin(env: Env) -> Result<Address, ContractError> {
        let admin_change_info: AdminChange = env
            .storage()
            .instance()
            .get(&PENDING_ADMIN)
            .unwrap_or_else(|| {
                log!(&env, "No admin change request is in place");
                panic_with_error!(&env, ContractError::NoAdminChangeInPlace);
            });

        let pending_admin = admin_change_info.new_admin;
        pending_admin.require_auth();

        if let Some(time_limit) = admin_change_info.time_limit {
            if env.ledger().timestamp() > time_limit {
                log!(&env, "Admin change expired");
                panic_with_error!(&env, ContractError::AdminChangeExpired);
            }
        }

        env.storage().instance().remove(&PENDING_ADMIN);

        save_admin_old(&env, &pending_admin);

        env.events()
            .publish(("Vesting: ", "Accepted new admin: "), &pending_admin);

        Ok(pending_admin)
    }
}

#[contractimpl]
impl Vesting {
    #[allow(dead_code)]
    //TODO: Remove after we've added the key to storage
    pub fn add_new_key_to_storage(env: Env) -> Result<(), ContractError> {
        env.storage().persistent().set(&VESTING_KEY, &true);
        Ok(())
    }

    #[allow(dead_code)]
    pub fn query_version(env: Env) -> String {
        String::from_str(&env, env!("CARGO_PKG_VERSION"))
    }
}
