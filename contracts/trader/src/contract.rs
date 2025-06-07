use phoenix::{
    ttl::{INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL},
    utils::AdminChange,
};
use soroban_sdk::{
    contract, contractimpl, contractmeta, log, panic_with_error, Address, BytesN, Env, String,
};

use crate::{
    error::ContractError,
    lp_contract,
    storage::{
        get_admin_old, get_name, get_output_token, get_pair, save_admin_old, save_name,
        save_output_token, save_pair, Asset, BalanceInfo, OutputTokenInfo, ADMIN, PENDING_ADMIN,
        TRADER_KEY,
    },
    token_contract,
};

contractmeta!(
    key = "Description",
    val = "Phoenix Protocol Designated Trader Contract"
);

#[contract]
pub struct Trader;

#[allow(dead_code)]
pub trait TraderTrait {
    fn trade_token(
        env: Env,
        sender: Address,
        token_to_swap: Address,
        liquidity_pool: Address,
        amount: Option<u64>,
        max_spread_bps: Option<i64>,
        deadline: Option<u64>,
        ask_asset_min_amount: Option<i128>,
        max_allowed_fee_bps: Option<i64>,
    );

    fn transfer(
        env: Env,
        sender: Address,
        recipient: Address,
        amount: i128,
        token_address: Option<Address>,
    );

    fn query_balances(env: Env) -> BalanceInfo;

    fn query_trading_pairs(env: Env) -> (Address, Address);

    fn query_admin_address(env: Env) -> Address;

    fn query_contract_name(env: Env) -> String;

    fn query_output_token_info(env: Env) -> OutputTokenInfo;

    fn update_contract_name(env: Env, new_name: String) -> Result<(), ContractError>;

    fn update_pair_addresses(env: Env, new_pair: (Address, Address)) -> Result<(), ContractError>;

    fn update_output_token(env: Env, new_token: Address) -> Result<(), ContractError>;

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
impl TraderTrait for Trader {
    fn trade_token(
        env: Env,
        sender: Address,
        token_to_swap: Address,
        liquidity_pool: Address,
        amount: Option<u64>,
        max_spread_bps: Option<i64>,
        deadline: Option<u64>,
        ask_asset_min_amount: Option<i128>,
        max_allowed_fee_bps: Option<i64>,
    ) {
        sender.require_auth();
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        if sender != get_admin_old(&env) {
            log!(&env, "Trader: Trade_token: Unauthorized trade");
            panic_with_error!(env, ContractError::Unauthorized);
        }

        if max_spread_bps.is_some()
            && (max_spread_bps.unwrap() < 0 || max_spread_bps.unwrap() > 10000)
        {
            log!(
                &env,
                "Trader: Trade token: Invalid max spread bps: {}",
                max_spread_bps.unwrap()
            );
            panic_with_error!(env, ContractError::InvalidMaxSpreadBps);
        }

        let (token_a, token_b) = get_pair(&env);
        if token_to_swap != token_a && token_to_swap != token_b {
            log!(
                &env,
                "Trader: Trade_token: Token to swap is not part of the trading pair: {}",
                token_to_swap
            );
            panic_with_error!(env, ContractError::SwapTokenNotInPair);
        }

        // TODO: this calls normal liquidity pool, we should know if it's a stable pool
        let lp_client = lp_contract::Client::new(&env, &liquidity_pool);
        let token_client = token_contract::Client::new(&env, &token_to_swap);

        let amount = if let Some(amount) = amount {
            amount as i128
        } else {
            token_client.balance(&sender)
        };

        let amount_swapped = lp_client.swap(
            &env.current_contract_address(),
            &token_to_swap,
            &amount,
            &ask_asset_min_amount,
            &max_spread_bps,
            &deadline,
            &max_allowed_fee_bps,
        );

        env.events()
            .publish(("Trader: Trade Token", "user: "), &sender);
        env.events()
            .publish(("Trader: Trade Token", "offer asset: "), &token_to_swap);
        env.events()
            .publish(("Trader: Trade Token", "amount received: "), amount_swapped);
    }

    fn transfer(
        env: Env,
        sender: Address,
        recipient: Address,
        amount: i128,
        token_address: Option<Address>,
    ) {
        sender.require_auth();
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        if sender != get_admin_old(&env) {
            log!(&env, "Trader: Transfer: Unauthorized transfer");
            panic_with_error!(env, ContractError::Unauthorized);
        }

        // If token_address is None, use the output token address to send to the recipient
        let token_address = match token_address {
            Some(token_address) => token_address,
            None => get_output_token(&env),
        };

        let token_client = token_contract::Client::new(&env, &token_address);

        token_client.transfer(&env.current_contract_address(), &recipient, &amount);
    }

    fn query_balances(env: Env) -> BalanceInfo {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);
        let output_token = get_output_token(&env);
        let (token_a, token_b) = get_pair(&env);

        let output_token_client = token_contract::Client::new(&env, &output_token);
        let token_a_client = token_contract::Client::new(&env, &token_a);
        let token_b_client = token_contract::Client::new(&env, &token_b);

        let output_token_balance = output_token_client.balance(&env.current_contract_address());
        let output_token_symbol = output_token_client.symbol();
        let token_a_balance = token_a_client.balance(&env.current_contract_address());
        let token_a_symbol = token_a_client.symbol();
        let token_b_balance = token_b_client.balance(&env.current_contract_address());
        let token_b_symbol = token_b_client.symbol();

        BalanceInfo {
            output_token: Asset {
                symbol: output_token_symbol,
                amount: output_token_balance,
            },
            token_a: Asset {
                symbol: token_a_symbol,
                amount: token_a_balance,
            },
            token_b: Asset {
                symbol: token_b_symbol,
                amount: token_b_balance,
            },
        }
    }

    fn query_trading_pairs(env: Env) -> (Address, Address) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);
        get_pair(&env)
    }

    fn query_admin_address(env: Env) -> Address {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);
        get_admin_old(&env)
    }

    fn query_contract_name(env: Env) -> String {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);
        get_name(&env)
    }

    fn query_output_token_info(env: Env) -> OutputTokenInfo {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);
        let output_token = get_output_token(&env);
        let output_token_client = token_contract::Client::new(&env, &output_token);

        OutputTokenInfo {
            address: output_token,
            name: output_token_client.name(),
            symbol: output_token_client.symbol(),
            decimal: output_token_client.decimals(),
        }
    }

    fn update_contract_name(env: Env, new_name: String) -> Result<(), ContractError> {
        get_admin_old(&env).require_auth();

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        save_name(&env, &new_name);

        env.events()
            .publish(("Trader: Update Name", "old:"), new_name);

        Ok(())
    }

    fn update_pair_addresses(env: Env, new_pair: (Address, Address)) -> Result<(), ContractError> {
        get_admin_old(&env).require_auth();

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        let output_token = get_output_token(&env);

        if output_token == new_pair.0 || output_token == new_pair.1 {
            log!(
                &env,
                "Trader: Update Pair: New pair addresses cannot include the output token"
            );
            panic_with_error!(env, ContractError::OutputTokenInPair);
        }

        save_pair(&env, &new_pair);

        env.events()
            .publish(("Trader: Update Pair", "old:"), new_pair);

        Ok(())
    }

    fn update_output_token(env: Env, new_token: Address) -> Result<(), ContractError> {
        get_admin_old(&env).require_auth();

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        let current_pair = get_pair(&env);
        if new_token == current_pair.0 || new_token == current_pair.1 {
            log!(
                &env,
                "Trader: Update Output Token: New token cannot be one of the pair addresses"
            );
            panic_with_error!(env, ContractError::OutputTokenInPair);
        }

        save_output_token(&env, &new_token);

        env.events()
            .publish(("Trader: Update Output Token", "old:"), new_token);

        Ok(())
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
            ("Trader: ", "Admin replacement requested by old admin: "),
            &current_admin,
        );
        env.events()
            .publish(("Trader: ", "Replace with new admin: "), &new_admin);

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
            .publish(("Trader: ", "Accepted new admin: "), &pending_admin);

        Ok(pending_admin)
    }
}

#[contractimpl]
impl Trader {
    pub fn __constructor(
        env: Env,
        admin: Address,
        contract_name: String,
        pair_addresses: (Address, Address),
        output_token: Address,
    ) {
        admin.require_auth();

        if output_token == pair_addresses.0 || output_token == pair_addresses.1 {
            log!(
                &env,
                "Trader: Update Pair: New pair addresses cannot include the output token"
            );
            panic_with_error!(env, ContractError::OutputTokenInPair);
        }

        save_admin_old(&env, &admin);

        save_name(&env, &contract_name);

        save_pair(&env, &pair_addresses);

        save_output_token(&env, &output_token);

        env.storage().persistent().set(&TRADER_KEY, &true);

        env.events()
            .publish(("Trader: Initialize", "admin: "), &admin);
        env.events()
            .publish(("Trader: Initialize", "contract name: "), contract_name);
        env.events()
            .publish(("Trader: Initialize", "pairs: "), pair_addresses);
        env.events()
            .publish(("Trader: Initialize", "PHO token: "), output_token);
    }

    #[allow(dead_code)]
    pub fn update(env: Env, new_wasm_hash: BytesN<32>) {
        let admin = get_admin_old(&env);
        admin.require_auth();

        env.deployer().update_current_contract_wasm(new_wasm_hash);
    }

    #[allow(dead_code)]
    pub fn query_version(env: Env) -> String {
        String::from_str(&env, env!("CARGO_PKG_VERSION"))
    }

    #[allow(dead_code)]
    #[cfg(not(tarpaulin_include))]
    //TODO: Remove after we've added the key to storage
    pub fn add_new_key_to_storage(env: Env) -> Result<(), ContractError> {
        env.storage().persistent().set(&TRADER_KEY, &true);
        Ok(())
    }
}
