use soroban_sdk::{
    contracttype, log, panic_with_error, symbol_short, xdr::ToXdr, Address, Bytes, BytesN,
    ConversionError, Env, Symbol, TryFromVal, Val,
};

use crate::{error::ContractError, token_contract};
use decimal::Decimal;

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
    /// The maximum amount of spread (in bps) that is tolerated during swap
    pub max_allowed_spread_bps: i64,
}
const CONFIG: Symbol = symbol_short!("CONFIG");

const MAX_TOTAL_FEE_BPS: i64 = 10_000;

/// This method is used to check fee bps.
pub fn validate_fee_bps(env: &Env, total_fee_bps: i64) -> i64 {
    if total_fee_bps > MAX_TOTAL_FEE_BPS {
        log!(
            env,
            "Stable Pool: Validate fee bps: Total fees cannot be greater than 100%"
        );
        panic_with_error!(
            env,
            ContractError::ValidateFeeBpsTotalFeesCantBeGreaterThan100
        );
    }
    total_fee_bps
}

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

pub fn get_greatest_precision(env: &Env) -> i32 {
    env.storage()
        .instance()
        .get(&DataKey::MaxPrecision)
        .unwrap()
}

pub fn save_greatest_precision(env: &Env, token1: &Address, token2: &Address) {
    let precision1 = token_contract::Client::new(env, token1).decimals();
    let precision2 = token_contract::Client::new(env, token2).decimals();
    let max_precision: u32 = if precision1 > precision2 {
        precision1
    } else {
        precision2
    };
    env.storage()
        .instance()
        .set(&DataKey::MaxPrecision, &i32::try_from(max_precision).ok());
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AmplifierParameters {
    pub init_amp: u64,
    pub init_amp_time: u64,
    pub next_amp: u64,
    pub next_amp_time: u64,
}

pub fn get_amp(env: &Env) -> Option<AmplifierParameters> {
    env.storage().instance().get(&DataKey::Amp)
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
    fn test_validate_fee_bps() {
        let env = Env::default();
        let result = validate_fee_bps(&env, 0);
        assert_eq!(result, 0);
        let result = validate_fee_bps(&env, 9999);
        assert_eq!(result, 9999);
        let result = validate_fee_bps(&env, 10_000);
        assert_eq!(result, 10_000);
    }

    #[test]
    #[should_panic(
        expected = "Stable Pool: Validate fee bps: Total fees cannot be greater than 100%"
    )]
    fn test_invalidate_fee_bps() {
        let env = Env::default();
        validate_fee_bps(&env, 10_001);
    }
}
