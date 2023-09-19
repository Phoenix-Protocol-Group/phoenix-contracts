use soroban_sdk::{contract, contractimpl, contractmeta, Address, Env, Vec, Symbol, Val, IntoVal};

use crate::error::ContractError;
use crate::storage::{get_liquidity_pool, save_admin, save_liquidity_pool, Pair, Swap};

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

    fn swap(env: Env, operations: Vec<Swap>, factory: Address) -> Result<(), ContractError>;
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
            let lp_address = lp.1;
            save_liquidity_pool(&env, pair, lp_address);
        }

        env.events()
            .publish(("initialize", "Multihop factory"), admin);

        Ok(())
    }

    fn swap(env: Env, operations: Vec<Swap>, factory: Address) -> Result<(), ContractError> {
        for op in operations.iter() {
            // few of questions
            // to get the liquidity_pool addr we need to query the factory. Where does the factory addr comes from?
            // currently Swap has the initial swap amount inside the struct. I guess we should get rid of it
            // are we supposed to call pair::swap() method, if yes, what would the rest of the values be? None?

            // might not be idea, but I want to code
            let init_fn_args: Vec<Val> = (
                // whos the sender?
                op.ask_asset.clone(),
                op.amount,
                None,
                None,
            )
                .into_val(&env);
            //        env: Env,
            //         sender: Address,
            //         sell_a: bool,
            //         offer_amount: i128,
            //         belief_price: Option<i64>,
            //         max_spread_bps: Option<i64>,
            env.invoke_contract(&factory, &Symbol::new(&env, "swap"), init_fn_args);
            let _lp_address = get_liquidity_pool(
                &env,
                Pair {
                    token_a: op.ask_asset,
                    token_b: op.offer_asset,
                },
            )?;
        }

        unimplemented!();
    }
}
