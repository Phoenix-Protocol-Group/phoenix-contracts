use phoenix::ttl::{INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL};
use soroban_sdk::{
    contract, contractimpl, contractmeta, log, panic_with_error, Address, BytesN, Env, String,
};

use crate::{
    error::ContractError,
    lp_contract,
    storage::{
        get_admin_old, get_name, get_output_token, get_pair, is_initialized, save_admin_old,
        save_name, save_output_token, save_pair, set_initialized, Asset, BalanceInfo,
        OutputTokenInfo, ADMIN, TRADER_KEY,
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
    fn initialize(
        env: Env,
        admin: Address,
        contract_name: String,
        pair_addresses: (Address, Address),
        output_token: Address,
    );

    #[allow(clippy::too_many_arguments)]
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

    fn migrate_admin_key(env: Env) -> Result<(), ContractError>;
}

#[contractimpl]
impl TraderTrait for Trader {
    fn initialize(
        env: Env,
        admin: Address,
        contract_name: String,
        pair_addresses: (Address, Address),
        output_token: Address,
    ) {
        admin.require_auth();

        if is_initialized(&env) {
            log!(&env, "Trader: Initialize: Cannot initialize trader twice!");
            panic_with_error!(env, ContractError::AlreadyInitialized)
        }

        save_admin_old(&env, &admin);

        save_name(&env, &contract_name);

        save_pair(&env, &pair_addresses);

        save_output_token(&env, &output_token);

        set_initialized(&env);

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

    #[allow(clippy::too_many_arguments)]
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

    fn migrate_admin_key(env: Env) -> Result<(), ContractError> {
        let admin = get_admin_old(&env);
        env.storage().instance().set(&ADMIN, &admin);

        Ok(())
    }
}

#[contractimpl]
impl Trader {
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
    //TODO: Remove after we've added the key to storage
    pub fn add_new_key_to_storage(env: Env) -> Result<(), ContractError> {
        env.storage().persistent().set(&TRADER_KEY, &true);
        Ok(())
    }
}
