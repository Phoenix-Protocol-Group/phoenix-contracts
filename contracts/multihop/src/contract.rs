use soroban_sdk::{
    contract, contractimpl, contractmeta, Address, BytesN, Env, IntoVal, Symbol, Val, Vec,
};

use crate::error::ContractError;
use crate::storage::{get_factory, save_admin, save_factory, Pair, Swap};

// Metadata that is added on to the WASM custom section
contractmeta!(
    key = "Description",
    val = "Contract to enable chaining of multiple swap transactions together"
);

#[contract]
pub struct Multihop;

pub trait MultihopTrait {
    fn initialize(
        env: Env,
        admin: Address,
        swap_info: Vec<(Pair, Address)>,
    ) -> Result<(), ContractError>;

    fn swap(
        env: Env,
        recipient: Address,
        operations: Vec<Swap>,
        amount: i128,
    ) -> Result<(), ContractError>;
}

#[contractimpl]
impl MultihopTrait for Multihop {
    fn initialize(
        env: Env,
        admin: Address,
        liquidity_pools: Vec<(Pair, Address)>,
    ) -> Result<(), ContractError> {
        save_admin(&env, &admin);

        for lp in liquidity_pools.iter() {
            let pair = lp.0;
            let factory = lp.1;
            save_factory(&env, pair, factory);
        }

        env.events()
            .publish(("initialize", "Multihop factory"), admin);

        Ok(())
    }

    fn swap(
        env: Env,
        recipient: Address,
        operations: Vec<Swap>,
        amount: i128,
    ) -> Result<(), ContractError> {
        // todo: use iterator afterwards
        if operations.is_empty() {
            return Err(ContractError::OperationsEmpty);
        }

        let mut asked_amount: i128 = amount;

        // this value will be updated in the iterator. Using from_contract_id as a placeholder
        let mut asked_token_addr: Address =
            Address::from_contract_id(&BytesN::from_array(&env, &[1u8; 0x20]));

        for op in operations.iter() {
            let current_pair = Pair {
                token_a: op.offer_asset.clone(),
                token_b: op.ask_asset.clone(),
            };

            let factory = get_factory(&env, current_pair)?;

            let factory_func_name = Symbol::new(&env, "query_for_pool_by_pair_tuple");
            let factory_call_args: Vec<Val> =
                (op.offer_asset.clone(), op.ask_asset.clone()).into_val(&env);
            let liquidity_pool_addr: Address =
                env.invoke_contract(&factory, &factory_func_name, factory_call_args);

            let lp_call_args: Vec<Val> = (
                env.current_contract_address(),
                op.offer_asset,
                asked_amount,
                None::<i64>,
                1i64,
            )
                .into_val(&env);
            let swap_fn: Symbol = Symbol::new(&env, "swap");
            env.invoke_contract::<Val>(&liquidity_pool_addr, &swap_fn, lp_call_args);

            let token_func_name = &Symbol::new(&env, "balance");
            let token_call_args: Vec<Val> = (env.current_contract_address(),).into_val(&env);
            asked_amount =
                env.invoke_contract(&op.ask_asset.clone(), token_func_name, token_call_args);
            asked_token_addr = op.ask_asset.clone();
        }

        let token_func_name = &Symbol::new(&env, "transfer");
        //from: Address, to: Address, amount: i128
        let token_call_args: Vec<Val> =
            (env.current_contract_address(), recipient, asked_amount).into_val(&env);
        env.invoke_contract::<Val>(&asked_token_addr, token_func_name, token_call_args);

        Ok(())
        // invoke_contract for the last token that was called
        // transfer the last asked_amount to the user
        // the user is the sender from input parameters
    }
}
