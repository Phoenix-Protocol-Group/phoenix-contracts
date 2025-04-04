use phoenix::ttl::{INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL};
use soroban_sdk::{
    contracttype, symbol_short, xdr::ToXdr, Address, Bytes, BytesN, ConversionError, Env, Symbol,
    TryFromVal, Val,
};

use crate::token_contract;
use soroban_decimal::Decimal;

pub const ADMIN: Symbol = symbol_short!("ADMIN");
pub const STABLE_POOL_KEY: Symbol = symbol_short!("STABLE_P");
pub(crate) const PENDING_ADMIN: Symbol = symbol_short!("p_admin");
const CONFIG: Symbol = symbol_short!("CONFIG");

#[derive(Clone, Copy)]
#[repr(u32)]
pub enum DataKey {
    TotalShares = 0,
    ReserveA = 1,
    ReserveB = 2,
    Admin = 3,
    Initialized = 4, // TODO: deprecated, remove in future upgrade
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

impl Config {
    pub fn protocol_fee_rate(&self) -> Decimal {
        Decimal::bps(self.total_fee_bps)
    }

    pub fn max_allowed_slippage(&self) -> Decimal {
        Decimal::bps(self.max_allowed_slippage_bps)
    }
}

pub fn get_config(env: &Env) -> Config {
    let config = env.storage().instance().get(&CONFIG).unwrap();
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

    config
}

pub fn save_config(env: &Env, config: Config) {
    env.storage().instance().set(&CONFIG, &config);
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);
}

pub fn get_greatest_precision(env: &Env) -> u32 {
    let greatest_precision = env
        .storage()
        .instance()
        .get(&DataKey::MaxPrecision)
        .unwrap();
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

    greatest_precision
}

pub fn get_precisions(env: &Env, token: &Address) -> u32 {
    let precision = env
        .storage()
        .instance()
        .get(&(DataKey::TokenPrecision, token))
        .unwrap();
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

    precision
}

pub fn save_greatest_precision(env: &Env, token1: &Address, token2: &Address) -> u32 {
    let precision1 = token_contract::Client::new(env, token1).decimals();
    let precision2 = token_contract::Client::new(env, token2).decimals();
    let max_precision: u32 = if precision1 > precision2 {
        precision1
    } else {
        precision2
    };
    env.storage()
        .instance()
        .set(&DataKey::MaxPrecision, &max_precision);
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

    env.storage()
        .instance()
        .set(&(DataKey::TokenPrecision, token1), &precision1);
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

    env.storage()
        .instance()
        .set(&(DataKey::TokenPrecision, token2), &precision2);
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

    max_precision
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
    let amp = env.storage().instance().get(&DataKey::Amp).unwrap();
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

    amp
}

pub fn save_amp(env: &Env, amp: AmplifierParameters) {
    env.storage().instance().set(&DataKey::Amp, &amp);
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);
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

    use phoenix::ttl::{INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL};
    use soroban_sdk::String;
    use soroban_sdk::{log, panic_with_error};

    use crate::error::ContractError;

    use super::*;

    #[allow(clippy::too_many_arguments)]
    pub fn deploy_token_contract(
        e: &Env,
        token_wasm_hash: BytesN<32>,
        token_a: &Address,
        token_b: &Address,
        admin: Address,
        decimals: u32,
        name: String,
        symbol: String,
    ) -> Address {
        let mut salt = Bytes::new(e);
        salt.append(&token_a.to_xdr(e));
        salt.append(&token_b.to_xdr(e));
        let salt = e.crypto().sha256(&salt);
        e.deployer()
            .with_current_contract(salt)
            .deploy_v2(token_wasm_hash, (admin, decimals, name, symbol))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn deploy_stake_contract(
        e: &Env,
        stake_wasm_hash: BytesN<32>,
        admin: &Address,
        share_token_address: &Address,
        min_bond: i128,
        min_reward: i128,
        manager: &Address,
        factory_addr: &Address,
        max_complexity: u32,
    ) -> Address {
        let salt = Bytes::new(e);
        let salt = e.crypto().sha256(&salt);

        e.deployer().with_current_contract(salt).deploy_v2(
            stake_wasm_hash,
            (
                admin,
                share_token_address,
                min_bond,
                min_reward,
                manager,
                factory_addr,
                max_complexity,
            ),
        )
    }

    pub fn save_admin_old(e: &Env, address: Address) {
        e.storage().instance().set(&DataKey::Admin, &address);
        e.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);
    }

    #[cfg(not(tarpaulin_include))]
    pub fn _save_admin(e: &Env, address: Address) {
        e.storage().instance().set(&ADMIN, &address);
        e.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);
    }

    pub fn save_total_shares(e: &Env, amount: i128) {
        e.storage().instance().set(&DataKey::TotalShares, &amount);
        e.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);
    }

    pub fn save_pool_balance_a(e: &Env, amount: i128) {
        e.storage().instance().set(&DataKey::ReserveA, &amount);
        e.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);
    }

    pub fn save_pool_balance_b(e: &Env, amount: i128) {
        e.storage().instance().set(&DataKey::ReserveB, &amount);
        e.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);
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
    pub fn get_admin_old(e: &Env) -> Address {
        let admin = e.storage().instance().get(&DataKey::Admin).unwrap();
        e.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        admin
    }

    #[cfg(not(tarpaulin_include))]
    pub fn _get_admin(e: &Env) -> Address {
        e.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        e.storage().instance().get(&ADMIN).unwrap_or_else(|| {
            log!(e, "Stable Pool: Admin not set");
            panic_with_error!(&e, ContractError::AdminNotSet)
        })
    }

    pub fn get_total_shares(e: &Env) -> i128 {
        let total_shares = e.storage().instance().get(&DataKey::TotalShares).unwrap();
        e.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        total_shares
    }
    pub fn get_pool_balance_a(e: &Env) -> i128 {
        let balance_a = e.storage().instance().get(&DataKey::ReserveA).unwrap();
        e.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        balance_a
    }

    pub fn get_pool_balance_b(e: &Env) -> i128 {
        let balance_b = e.storage().instance().get(&DataKey::ReserveB).unwrap();
        e.storage()
            .instance()
            .extend_ttl(INSTANCE_RENEWAL_THRESHOLD, INSTANCE_TARGET_TTL);

        balance_b
    }

    pub fn get_balance(e: &Env, contract: &Address) -> i128 {
        token_contract::Client::new(e, contract).balance(&e.current_contract_address())
    }
}

#[cfg(test)]
mod tests {
    use soroban_sdk::testutils::Ledger;

    use crate::math::compute_current_amp;

    use super::*;

    #[test]
    #[should_panic]
    fn test_get_admin_failure() {
        let env = Env::default();
        let _ = utils::get_admin_old(&env);
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
    fn test_compute_current_amp() {
        let env = Env::default();
        let params1 = AmplifierParameters {
            init_amp: 100,
            init_amp_time: 0,
            next_amp: 200,
            next_amp_time: 100,
        };

        assert_eq!(compute_current_amp(&env, &params1), 100);
        let params2 = AmplifierParameters {
            init_amp: 100,
            init_amp_time: 0,
            next_amp: 200,
            next_amp_time: 100,
        };

        env.ledger().set_timestamp(100);
        assert_eq!(compute_current_amp(&env, &params2), 200);

        let params3 = AmplifierParameters {
            init_amp: 200,
            init_amp_time: 0,
            next_amp: 100,
            next_amp_time: 100,
        };

        env.ledger().set_timestamp(50);
        assert_eq!(compute_current_amp(&env, &params3), 150);
    }
}
