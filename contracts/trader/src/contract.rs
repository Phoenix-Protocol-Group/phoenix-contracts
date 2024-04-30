use decimal::Decimal;
use soroban_sdk::{contract, contractmeta, Address, Env, String};

use crate::storage::{save_admin, save_name, save_pair, save_spread, save_token};

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

    fn trade_token(env: Env, token_address: Address, liquidity_pool: Address, amount: Option<u64>);

    fn transfer(env: Env, recipient: Address, amount: u64, token_address: Option<Address>);
}

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

        save_token(&env, &pho_token);

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

    fn trade_token(env: Env, token_address: Address, liquidity_pool: Address, amount: Option<u64>) {
        todo!()
    }

    fn transfer(env: Env, recipient: Address, amount: u64, token_address: Option<Address>) {
        todo!()
    }
}
