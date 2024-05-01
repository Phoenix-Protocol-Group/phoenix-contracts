use soroban_sdk::{
    contract, contractimpl, contractmeta, log, panic_with_error, token, Address, Env, String,
};

use crate::{
    error::ContractError,
    lp_contract,
    storage::{
        get_admin, get_name, get_pair, get_pho_token, get_spread, save_admin, save_name, save_pair,
        save_pho_token, save_spread, BalanceInfo,
    },
    token_contract,
};

contractmeta!(
    key = "Description",
    val = "Phoenix Protocol Designated Trader Contract"
);

#[contract]
pub struct Trader;

pub trait TraderTrait {
    fn initialize(
        env: Env,
        admin: Address,
        contract_name: String,
        pair_addresses: (Address, Address),
        pho_token: Address,
        max_spread_bps: Option<u64>,
    );

    fn trade_token(
        env: Env,
        sender: Address,
        token_to_swap: Address,
        liquidity_pool: Address,
        amount: Option<u64>,
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
}

#[contractimpl]
impl TraderTrait for Trader {
    fn initialize(
        env: Env,
        admin: Address,
        contract_name: String,
        pair_addresses: (Address, Address),
        pho_token: Address,
        max_spread: Option<u64>,
    ) {
        admin.require_auth();

        save_admin(&env, &admin);

        save_name(&env, &contract_name);

        save_pair(&env, &pair_addresses);

        save_pho_token(&env, &pho_token);

        if let Some(spread) = max_spread {
            save_spread(&env, &spread);
        }

        env.events()
            .publish(("Trader: Initialize", "admin: "), &admin);
        env.events()
            .publish(("Trader: Initialize", "contract name: "), contract_name);
        env.events()
            .publish(("Trader: Initialize", "pairs: "), pair_addresses);
        env.events()
            .publish(("Trader: Initialize", "PHO token: "), pho_token);
    }

    fn trade_token(
        env: Env,
        sender: Address,
        token_to_swap: Address,
        liquidity_pool: Address,
        amount: Option<u64>,
    ) {
        sender.require_auth();

        if sender != get_admin(&env) {
            log!(&env, "Unauthorized");
            panic_with_error!(env, ContractError::Unauthorized);
        }

        let (token_a, token_b) = get_pair(&env);
        if token_to_swap != token_a && token_to_swap != token_b {
            log!(
                &env,
                "Token to swap is not part of the trading pair: {}",
                token_to_swap
            );
            panic_with_error!(env, ContractError::SwapTokenNotInPair);
        }

        let lp_client = lp_contract::Client::new(&env, &liquidity_pool);
        let token_client = token_contract::Client::new(&env, &token_to_swap);

        let amount = if let Some(amount) = amount {
            amount as i128
        } else {
            token_client.balance(&sender)
        };

        let max_spread_bps = get_spread(&env);

        let amount_swapped = lp_client.swap(
            &env.current_contract_address(),
            &token_to_swap,
            &amount,
            &None,
            &Some(max_spread_bps as i64),
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

        if sender != get_admin(&env) {
            log!(&env, "Unauthorized");
            panic_with_error!(env, ContractError::Unauthorized);
        }

        // If token_address is None, use the PHO token address to send to the recipient
        let token_address = match token_address {
            Some(token_address) => token_address,
            None => get_pho_token(&env),
        };

        let token_client = token_contract::Client::new(&env, &token_address);

        token_client.transfer(&env.current_contract_address(), &recipient, &amount);
    }

    fn query_balances(env: Env) -> BalanceInfo {
        let pho_token = get_pho_token(&env);
        let (token_a, token_b) = get_pair(&env);

        let pho_token_client = token_contract::Client::new(&env, &pho_token);
        let token_a_client = token_contract::Client::new(&env, &token_a);
        let token_b_client = token_contract::Client::new(&env, &token_b);

        let pho_balance = pho_token_client.balance(&env.current_contract_address());
        let token_a_balance = token_a_client.balance(&env.current_contract_address());
        let token_b_balance = token_b_client.balance(&env.current_contract_address());

        BalanceInfo {
            pho: pho_balance,
            token_a: token_a_balance,
            token_b: token_b_balance,
        }
    }

    fn query_trading_pairs(env: Env) -> (Address, Address) {
        get_pair(&env)
    }

    fn query_admin_address(env: Env) -> Address {
        get_admin(&env)
    }

    fn query_contract_name(env: Env) -> String {
        get_name(&env)
    }
}
