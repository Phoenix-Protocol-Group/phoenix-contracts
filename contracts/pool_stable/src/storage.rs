use soroban_sdk::{
    contracttype, log, panic_with_error, symbol_short, xdr::ToXdr, Address, Bytes, BytesN,
    ConversionError, Env, Symbol, TryFromVal, Val,
};

use crate::{error::ContractError, token_contract, MAXIMUM_ALLOWED_PRECISION};
use soroban_decimal::Decimal;

#[derive(Clone, Copy)]
#[repr(u32)]
pub enum DataKey {
    TotalShares = 0,
    ReserveA = 1,
    ReserveB = 2,
    Admin = 3,
    Initialized = 4,
    Amp = 5,
    MaxPrecision = 6,
    TokenPrecision = 7,
}

impl TryFromVal<Env, DataKey> for Val {
    type Error = ConversionError;

    fn try_from_val(_env: &Env, v: &DataKey) -> Result<Self, Self::Error> {
        Ok((*v as u32).into())
    }
}

#[contracttype]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum PairType {
    Xyk = 0,
    Stable = 1,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    pub token_a: Address,
    pub token_b: Address,
    pub share_token: Address,
    pub stake_contract: Address,
    pub pool_type: PairType,
    /// The total fees (in bps) charged by a pool of this type.
    /// In relation to the returned amount of tokens
    pub total_fee_bps: i64,
    pub fee_recipient: Address,
    /// The maximum amount of slippage (in bps) that is tolerated during providing liquidity
    pub max_allowed_slippage_bps: i64,
    /// Default slippage, in case the customer hasn't specified
    pub default_slippage_bps: i64,
    /// The maximum amount of spread (in bps) that is tolerated during swap
    pub max_allowed_spread_bps: i64,
}
const CONFIG: Symbol = symbol_short!("CONFIG");

impl Config {
    pub fn protocol_fee_rate(&self) -> Decimal {
        Decimal::bps(self.total_fee_bps)
    }

    pub fn max_allowed_slippage(&self) -> Decimal {
        Decimal::bps(self.max_allowed_slippage_bps)
    }
}

pub fn get_config(env: &Env) -> Config {
    env.storage().instance().get(&CONFIG).unwrap()
}

pub fn save_config(env: &Env, config: Config) {
    env.storage().instance().set(&CONFIG, &config);
}

pub fn get_greatest_precision(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&DataKey::MaxPrecision)
        .unwrap()
}

pub fn get_precisions(env: &Env, token: &Address) -> u32 {
    env.storage()
        .instance()
        .get(&(DataKey::TokenPrecision, token))
        .unwrap()
}

pub fn save_greatest_precision(env: &Env, token1: &Address, token2: &Address) {
    let precision1 = token_contract::Client::new(env, token1).decimals();
    let precision2 = token_contract::Client::new(env, token2).decimals();

    verify_precision(env, precision1, precision2);

    // NOTE: now that we must have tokens with equal number of decimals, this isn't needed
    // let max_precision: u32 = if precision1 > precision2 {
    //     precision1
    // } else {
    //     precision2
    // };

    env.storage()
        .instance()
        .set(&DataKey::MaxPrecision, &precision1);
    env.storage()
        .instance()
        .set(&(DataKey::TokenPrecision, token1), &precision1);
    env.storage()
        .instance()
        .set(&(DataKey::TokenPrecision, token2), &precision2);
}

fn verify_precision(env: &Env, p1: u32, p2: u32) {
    if p1 > MAXIMUM_ALLOWED_PRECISION || p2 > MAXIMUM_ALLOWED_PRECISION {
        log!(&env, "Pool Stable: Initialize: precision above the limit");
        panic_with_error!(env, ContractError::MaximumAllowedPrecisionViolated);
    }

    if p1 != p2 {
        log!(&env, "Pool Stable: Initialize: precision missmatch");
        panic_with_error!(env, ContractError::PresicionMissmatch);
    }
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AmplifierParameters {
    pub init_amp: u64,
    pub init_amp_time: u64,
    pub next_amp: u64,
    pub next_amp_time: u64,
}

pub fn get_amp(env: &Env) -> AmplifierParameters {
    env.storage().instance().get(&DataKey::Amp).unwrap()
}

pub fn save_amp(env: &Env, amp: AmplifierParameters) {
    env.storage().instance().set(&DataKey::Amp, &amp);
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Asset {
    /// Address of the asset
    pub address: Address,
    /// The total amount of those tokens in the pool
    pub amount: i128,
}

/// This struct is used to return a query result with the total amount of LP tokens and assets in a specific pool.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PoolResponse {
    /// The asset A in the pool together with asset amounts
    pub asset_a: Asset,
    /// The asset B in the pool together with asset amounts
    pub asset_b: Asset,
    /// The total amount of LP tokens currently issued
    pub asset_lp_share: Asset,
    /// The address of the Stake contract for the liquidity pool
    pub stake_address: Address,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StableLiquidityPoolInfo {
    pub pool_address: Address,
    pub pool_response: PoolResponse,
    pub total_fee_bps: i64,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SimulateSwapResponse {
    pub ask_amount: i128,
    pub commission_amount: i128,
    pub spread_amount: i128,
    pub total_return: i128,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SimulateReverseSwapResponse {
    pub offer_amount: i128,
    pub commission_amount: i128,
    pub spread_amount: i128,
}

pub mod utils {
    use super::*;

    pub fn deploy_token_contract(
        e: &Env,
        token_wasm_hash: BytesN<32>,
        token_a: &Address,
        token_b: &Address,
    ) -> Address {
        let mut salt = Bytes::new(e);
        salt.append(&token_a.to_xdr(e));
        salt.append(&token_b.to_xdr(e));
        let salt = e.crypto().sha256(&salt);
        e.deployer()
            .with_current_contract(salt)
            .deploy(token_wasm_hash)
    }

    pub fn deploy_stake_contract(e: &Env, stake_wasm_hash: BytesN<32>) -> Address {
        let salt = Bytes::new(e);
        let salt = e.crypto().sha256(&salt);

        e.deployer()
            .with_current_contract(salt)
            .deploy(stake_wasm_hash)
    }

    pub fn save_admin(e: &Env, address: Address) {
        e.storage().instance().set(&DataKey::Admin, &address)
    }

    pub fn save_total_shares(e: &Env, amount: i128) {
        e.storage().instance().set(&DataKey::TotalShares, &amount)
    }

    pub fn save_pool_balance_a(e: &Env, amount: i128) {
        e.storage().instance().set(&DataKey::ReserveA, &amount)
    }

    pub fn save_pool_balance_b(e: &Env, amount: i128) {
        e.storage().instance().set(&DataKey::ReserveB, &amount)
    }

    pub fn mint_shares(e: &Env, share_token: &Address, to: &Address, amount: i128) {
        let total = get_total_shares(e);

        token_contract::Client::new(e, share_token).mint(to, &amount);

        save_total_shares(e, total + amount);
    }

    pub fn burn_shares(e: &Env, share_token: &Address, amount: i128) {
        let total = get_total_shares(e);

        token_contract::Client::new(e, share_token).burn(&e.current_contract_address(), &amount);

        save_total_shares(e, total - amount);
    }

    // queries
    pub fn get_admin(e: &Env) -> Address {
        e.storage().instance().get(&DataKey::Admin).unwrap()
    }

    pub fn get_total_shares(e: &Env) -> i128 {
        e.storage().instance().get(&DataKey::TotalShares).unwrap()
    }
    pub fn get_pool_balance_a(e: &Env) -> i128 {
        e.storage().instance().get(&DataKey::ReserveA).unwrap()
    }

    pub fn get_pool_balance_b(e: &Env) -> i128 {
        e.storage().instance().get(&DataKey::ReserveB).unwrap()
    }

    pub fn get_balance(e: &Env, contract: &Address) -> i128 {
        token_contract::Client::new(e, contract).balance(&e.current_contract_address())
    }

    pub fn is_initialized(e: &Env) -> bool {
        e.storage()
            .persistent()
            .get(&DataKey::Initialized)
            .unwrap_or(false)
    }

    pub fn set_initialized(e: &Env) {
        e.storage().persistent().set(&DataKey::Initialized, &true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_get_admin_failure() {
        let env = Env::default();
        let _ = utils::get_admin(&env);
    }

    #[test]
    #[should_panic]
    fn test_get_total_shares_failure() {
        let env = Env::default();
        let _ = utils::get_total_shares(&env);
    }

    #[test]
    #[should_panic]
    fn test_get_pool_balance_a_failure() {
        let env = Env::default();
        let _ = utils::get_pool_balance_a(&env);
    }

    #[test]
    #[should_panic]
    fn test_get_pool_balance_b_failure() {
        let env = Env::default();
        let _ = utils::get_pool_balance_b(&env);
    }

    #[test]
    #[should_panic(expected = "Pool Stable: Initialize: precision above the limit")]
    fn test_should_panic_when_precision_above_the_allowance_used() {
        let env = Env::default();
        verify_precision(&env, 8, 8);
    }

    #[test]
    #[should_panic(expected = "Pool Stable: Initialize: precision missmatch")]
    fn test_should_panic_when_different_precision_used() {
        let env = Env::default();
        verify_precision(&env, 5, 7);
    }
}
