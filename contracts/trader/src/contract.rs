use decimal::Decimal;
use soroban_sdk::{contract, contractmeta, Address, Env, String};

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
        max_spread: Option<Decimal>,
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
        max_spread: Option<Decimal>,
    ) {
        todo!()
    }

    fn trade_token(env: Env, token_address: Address, liquidity_pool: Address, amount: Option<u64>) {
        todo!()
    }

    fn transfer(env: Env, recipient: Address, amount: u64, token_address: Option<Address>) {
        todo!()
    }
}
