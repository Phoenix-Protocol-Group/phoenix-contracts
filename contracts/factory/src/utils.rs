use crate::error::ContractError;
use soroban_sdk::arbitrary::std::dbg;
use soroban_sdk::{Address, BytesN, Env, Symbol, Val, Vec};

pub fn deploy_lp_contract(
    env: &Env,
    lp_wasm_hash: BytesN<32>,
    salt: BytesN<32>,
    init_fn: Symbol,
    init_fn_args: Vec<Val>,
) -> Result<Address, ContractError> {
    let deployer = env.current_contract_address();

    if deployer != env.current_contract_address() {
        deployer.require_auth();
    }

    dbg!(&salt);
    let deployed_address = env
        .deployer()
        .with_address(deployer, salt)
        .deploy(lp_wasm_hash);
    dbg!("calling the deploy liquidity pool function twice, this should be twice");

    let res: Val = env.invoke_contract(&deployed_address, &init_fn, init_fn_args);

    if !res.is_void() {
        return Err(ContractError::ContractNotDeployed);
    }

    Ok(deployed_address)
}
