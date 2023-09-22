use soroban_sdk::{contract, contractimpl, contractmeta, Address, Env, IntoVal, Symbol, Val, Vec};

use crate::error::ContractError;
use crate::storage::{get_factory, save_admin, save_factory, Pair, PoolResponse, Swap};

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

    fn swap(env: Env, operations: Vec<Swap>, amount: i128) -> Result<(), ContractError>;
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

    fn swap(env: Env, operations: Vec<Swap>, amount: i128) -> Result<(), ContractError> {
        // todo: use iterator afterwards
        let mut asked_amount: i128 = amount;

        for op in operations.iter() {
            let current_pair = Pair {
                token_a: op.offer_asset.clone(),
                token_b: op.ask_asset.clone(),
            };

            let factory = get_factory(&env, current_pair)?;

            let factory_func_name = Symbol::new(&env, "query_for_pool_by_pair_tuple");
            let factory_call_args: Vec<Val> = (op.offer_asset.clone(), op.ask_asset).into_val(&env);
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
            // in pair contract the swap method returns Ok(())
            let res: Val = env.invoke_contract(&liquidity_pool_addr, &swap_fn, lp_call_args);

            // according to docs:
            // Invokes a function of a contract that is registered in the Env.
            // Panics
            // Will panic if the contract_id does not match a registered contract, func does not
            // match a function of the referenced contract, or the number of args do not match the
            //
            // argument count of the referenced contract function.

            // Will panic if the contract that is invoked fails or aborts in anyway.

            // Will panic if the value returned from the contract cannot be converted into the type T.

            // I don't think this is needed in this case
            if res.is_void() {
                return Err(ContractError::RemoteCallFailed);
            }

            // querying liquidity pool info again, because the swap method does not return amount left
            let lp_func_name = Symbol::new(&env, "query_pool_info");
            let lp_info: PoolResponse =
                env.invoke_contract(&liquidity_pool_addr, &lp_func_name, Vec::new(&env));

            // check the remaining amount of the asked asset
            let asked_asset_amount = lp_info.asset_b.amount;
            asked_amount = asked_asset_amount;

            // where do we send the final sum?
        }

        Err(ContractError::OperationsEmpty)
    }
}
