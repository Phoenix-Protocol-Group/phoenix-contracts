use soroban_sdk::{xdr::ToXdr, Address, Bytes, BytesN, Env};

pub fn deploy_lp_contract(
    env: &Env,
    wasm_hash: BytesN<32>,
    token_a: &Address,
    token_b: &Address,
) -> Address {
    let mut salt = Bytes::new(env);
    salt.append(&token_a.to_xdr(env));
    salt.append(&token_b.to_xdr(env));
    let salt = env.crypto().sha256(&salt);

    env.deployer()
        .with_current_contract(salt)
        .deploy_v2(wasm_hash, ())
}

pub fn deploy_and_initialize_multihop_contract(
    env: Env,
    admin: Address,
    multihop_wasm_hash: BytesN<32>,
) -> Address {
    let mut salt = Bytes::new(&env);
    salt.append(&admin.clone().to_xdr(&env));
    let salt = env.crypto().sha256(&salt);

    env.deployer()
        .with_current_contract(salt)
        .deploy_v2(multihop_wasm_hash, (admin, env.current_contract_address()))
}
