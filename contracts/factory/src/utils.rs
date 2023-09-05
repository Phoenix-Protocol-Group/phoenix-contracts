use soroban_sdk::{Address, Bytes, BytesN, Env};

pub fn deploy_lp_contract(env: &Env, lp_wasm_hash: BytesN<32>) -> Address {
    let deployer = env.current_contract_address();

    if deployer != env.current_contract_address() {
        deployer.require_auth();
    }

    let salt = Bytes::new(env);
    let salt = env.crypto().sha256(&salt);

    env.deployer()
        .with_address(deployer, salt)
        .deploy(lp_wasm_hash)
}
