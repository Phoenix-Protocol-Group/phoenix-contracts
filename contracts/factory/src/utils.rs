use soroban_sdk::{xdr::ToXdr, Address, Bytes, BytesN, Env};

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
