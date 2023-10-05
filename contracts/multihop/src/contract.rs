use soroban_sdk::{contract, contractimpl, contractmeta, Address, Env, IntoVal, Symbol, Val, Vec};

use crate::error::ContractError;
use crate::storage::{get_factory, save_admin, save_factory, Swap};

// Metadata that is added on to the WASM custom section
contractmeta!(
    key = "Description",
    val = "Contract to enable chaining of multiple swap transactions together"
);

#[contract]
pub struct Multihop;

pub trait MultihopTrait {
    fn initialize(env: Env, admin: Address, factory: Address) -> Result<(), ContractError>;

    fn swap(
        env: Env,
        recipient: Address,
        operations: Vec<Swap>,
        amount: i128,
    ) -> Result<(), ContractError>;
}

#[contractimpl]
impl MultihopTrait for Multihop {
    fn initialize(env: Env, admin: Address, factory: Address) -> Result<(), ContractError> {
        save_admin(&env, &admin);

        save_factory(&env, factory);

        env.events()
            .publish(("initialize", "Multihop factory with admin: "), admin);

        Ok(())
    }

    fn swap(
        env: Env,
        recipient: Address,
        operations: Vec<Swap>,
        amount: i128,
    ) -> Result<(), ContractError> {
        recipient.require_auth();

        if operations.is_empty() {
            return Err(ContractError::OperationsEmpty);
        }

        let mut offer_amount: i128 = amount;
        let mut offer_token_addr: Address = operations.get(0).unwrap().ask_asset.clone();

        // first transfer token to multihop contract
        let token_func_name = &Symbol::new(&env, "transfer");
        let token_call_args: Vec<Val> =
            (&recipient, env.current_contract_address(), offer_amount).into_val(&env);
        env.invoke_contract::<Val>(&offer_token_addr, token_func_name, token_call_args);

        operations.iter().for_each(|op| {
            let factory = get_factory(&env).expect("factory not found");

            let factory_func_name = Symbol::new(&env, "query_for_pool_by_pair_tuple");
            let factory_call_args: Vec<Val> =
                (op.offer_asset.clone(), op.ask_asset.clone()).into_val(&env);
            let liquidity_pool_addr: Address =
                env.invoke_contract(&factory, &factory_func_name, factory_call_args);

            let lp_call_args: Vec<Val> = (
                env.current_contract_address(),
                true,
                offer_amount,
                None::<i64>,
                Some(5000i64),
            )
                .into_val(&env);
            let swap_fn: Symbol = Symbol::new(&env, "swap");
            env.invoke_contract::<Val>(&liquidity_pool_addr, &swap_fn, lp_call_args);

            let token_func_name = &Symbol::new(&env, "balance");
            let token_call_args: Vec<Val> = (env.current_contract_address(),).into_val(&env);
            offer_amount =
                env.invoke_contract(&op.ask_asset, token_func_name, token_call_args);
            dbg!("balance after: {}", offer_amount);
            offer_token_addr = op.ask_asset.clone();
        });

        // dbg!("out of the loop");
        // in each loop iteration, last asked token becomes an offer; after loop we can rename it
        let asked_amount = offer_amount;
        let asked_token_addr = offer_token_addr;

        let token_func_name = &Symbol::new(&env, "transfer");
        let token_call_args: Vec<Val> =
            (env.current_contract_address(), recipient, asked_amount).into_val(&env);
        env.invoke_contract::<Val>(&asked_token_addr, token_func_name, token_call_args);

        Ok(())
    }
}
