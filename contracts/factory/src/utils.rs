use soroban_sdk::{xdr::ToXdr, Address, Bytes, BytesN, Env, IntoVal, Symbol, Val, Vec};

pub fn deploy_lp_contract(
    env: &Env,
    lp_wasm_hash: BytesN<32>,
    token_a: &Address,
    token_b: &Address,
) -> Address {
    let deployer = env.current_contract_address();

    if deployer != env.current_contract_address() {
        deployer.require_auth();
    }

    let mut salt = Bytes::new(env);
    salt.append(&token_a.to_xdr(env));
    salt.append(&token_b.to_xdr(env));
    let salt = env.crypto().sha256(&salt);

    env.deployer()
        .with_current_contract(salt)
        .deploy(lp_wasm_hash)
}

pub fn deploy_multihop_contract(
    env: Env,
    admin: Address,
    multihop_wasm_hash: BytesN<32>,
) -> Address {
    let mut salt = Bytes::new(&env);
    salt.append(&admin.clone().to_xdr(&env));
    let salt = env.crypto().sha256(&salt);

    let multihop_address = env
        .deployer()
        .with_current_contract(salt)
        .deploy(multihop_wasm_hash);

    let init_fn = Symbol::new(&env, "initialize");
    let init_args: Vec<Val> = (admin, env.current_contract_address()).into_val(&env);
    env.invoke_contract::<Val>(&multihop_address, &init_fn, init_args);

    multihop_address
}
