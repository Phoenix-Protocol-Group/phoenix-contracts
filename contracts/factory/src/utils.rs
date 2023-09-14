use soroban_sdk::{xdr::ToXdr, Address, Bytes, BytesN, Env};

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
