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
    /// The maximum allowed percentage (in bps) for referral fee
    pub max_referral_bps: i64,
}
const CONFIG: Symbol = symbol_short!("CONFIG");

const MAX_TOTAL_FEE_BPS: i64 = 10_000;

/// This method is used to check fee bps.
pub fn validate_fee_bps(env: &Env, total_fee_bps: i64) -> i64 {
    if total_fee_bps > MAX_TOTAL_FEE_BPS {
        log!(
            env,
            "Pool: Validate fee bps: Total fees cannot be greater than 100%"
        );
        panic_with_error!(
            env,
            ContractError::ValidateFeeBpsTotalFeesCantBeGreaterThen100
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

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Asset {
    /// Address of the asset
    pub address: Address,
    /// The total amount of those tokens in the pool
    pub amount: i128,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComputeSwap {
    /// The amount that will be returned to the user, after all fees and spread has been taken into
    /// account.
    pub return_amount: i128,
    /// The spread amount, that is the difference between expected and actual swap amount.
    pub spread_amount: i128,
    /// The commision amount is the fee that is charged by the pool for the swap service.
    pub commission_amount: i128,
    /// The referral fee is the fee that will be given back to the referral. `0` if no referral is
    /// set.
    pub referral_fee_amount: i128,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Referral {
    /// Address of the referral
    pub address: Address,
    /// fee in bps, later parsed to percentage
    pub fee: i64,
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
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiquidityPoolInfo {
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

    #[allow(clippy::too_many_arguments)]
    pub fn get_deposit_amounts(
        env: &Env,
        desired_a: i128,
        min_a: Option<i128>,
        desired_b: i128,
        min_b: Option<i128>,
        pool_balance_a: i128,
        pool_balance_b: i128,
        allowed_slippage: Decimal,
    ) -> (i128, i128) {
        if pool_balance_a == 0 && pool_balance_b == 0 {
            return (desired_a, desired_b);
        }

        if let Some(min_a) = min_a {
            if min_a > desired_a {
                panic_with_error!(env, ContractError::GetDepositAmountsMinABiggerThenDesiredA);
            }
        }
        if let Some(min_b) = min_b {
            if min_b > desired_b {
                panic_with_error!(env, ContractError::GetDepositAmountsMinABiggerThenDesiredA);
            }
        }

        let amount_a = {
            let mut amount_a = desired_b * pool_balance_a / pool_balance_b;
            if amount_a > desired_a {
                // If the amount is within the desired amount of slippage, we accept it
                if Decimal::from_ratio(amount_a, desired_a) - Decimal::one() <= allowed_slippage {
                    amount_a = desired_a;
                } else {
                    log!(
                        env,
                        "Pool: Get deposit amounts: Deposit amount for asset A ({}) is invalid. It exceeds the desired amount ({})",
                        amount_a,
                        desired_a,
                    );
                    panic_with_error!(
                        env,
                        ContractError::GetDepositAmountsAmountABiggerThenDesiredA
                    );
                }
            };
            if let Some(min_a) = min_a {
                if amount_a < min_a {
                    log!(
                        env,
                        "Pool: Get deposit amounts: Deposit amount for asset A ({}) is invalid. It falls below the minimum requirement ({})",
                        amount_a,
                        min_a
                    );
                    panic_with_error!(env, ContractError::GetDepositAmountsAmountALessThenMinA);
                }
            }
            amount_a
        };

        let amount_b = {
            let mut amount_b = desired_a * pool_balance_b / pool_balance_a;
            if amount_b > desired_b {
                // If the amount is within the set threshold of the desired amount, we accept it
                if Decimal::from_ratio(amount_b, desired_b) - Decimal::one() <= allowed_slippage {
                    amount_b = desired_b;
                } else {
                    log!(
                env,
                "Pool: Get deposit amounts: Deposit amount for asset B ({}) is invalid. It exceeds the desired amount ({})",
                amount_b,
                desired_b,
            );
                    panic_with_error!(
                        env,
                        ContractError::GetDepositAmountsAmountBBiggerThenDesiredB
                    );
                }
            };
            if let Some(min_b) = min_b {
                if amount_b < min_b {
                    log!(
                env,
                "Pool: Get deposit amounts: Deposit amount for asset B ({}) is invalid. It falls below the minimum requirement ({})",
                amount_b,
                min_b
            );
                    panic_with_error!(env, ContractError::GetDepositAmountsAmountBLessThenMinB);
                }
            }
            amount_b
        };

        (amount_a, amount_b)
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
    fn test_get_deposit_amounts_pool_balances_zero() {
        let env = Env::default();
        let result =
            utils::get_deposit_amounts(&env, 100, Some(50), 200, Some(50), 0, 0, Decimal::bps(100));
        assert_eq!(result, (100, 200));
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #12)")]
    fn test_get_deposit_amounts_amount_b_less_than_desired() {
        let env = Env::default();
        utils::get_deposit_amounts(&env, 1000, None, 1005, Some(1001), 1, 1, Decimal::bps(100));
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #12)")]
    fn test_get_deposit_amounts_amount_b_less_than_min_b() {
        let env = Env::default();
        utils::get_deposit_amounts(&env, 1000, None, 1005, Some(1001), 1, 1, Decimal::bps(100));
    }

    #[test]
    fn test_get_deposit_amounts_amount_a_less_than_desired_and_greater_than_min_a() {
        let env = Env::default();
        let result = utils::get_deposit_amounts(
            &env,
            100,
            Some(50),
            200,
            Some(150),
            100,
            200,
            Decimal::bps(100),
        );
        assert_eq!(result, (100, 200));
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #7)")]
    fn test_get_deposit_amounts_amount_a_greater_than_desired_and_less_than_min_a() {
        let env = Env::default();
        utils::get_deposit_amounts(&env, 50, Some(100), 200, None, 100, 200, Decimal::bps(100));
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #7)")]
    fn test_get_deposit_amounts_amount_b_greater_than_desired_and_less_than_min_b() {
        let env = Env::default();
        utils::get_deposit_amounts(
            &env,
            150,
            Some(100),
            200,
            Some(300),
            100,
            200,
            Decimal::bps(100),
        );
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #7)")]
    fn test_get_deposit_amounts_amount_a_less_than_min_a() {
        let env = Env::default();
        utils::get_deposit_amounts(&env, 100, Some(200), 200, None, 100, 200, Decimal::bps(100));
    }

    #[test]
    fn test_get_deposit_amounts_ratio() {
        let env = Env::default();
        let (amount_a, amount_b) = utils::get_deposit_amounts(
            &env,
            1000,
            None,
            2000,
            None,
            5000,
            10000,
            Decimal::bps(100),
        );
        // The desired ratio is within 1% of the current pool ratio, so the desired amounts are returned
        assert_eq!(amount_a, 1000);
        assert_eq!(amount_b, 2000);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #9)")]
    fn test_get_deposit_amounts_exceeds_desired() {
        let env = Env::default();
        // The calculated deposit for asset A exceeds the desired amount and is not within 1% tolerance
        utils::get_deposit_amounts(&env, 1000, None, 2000, None, 10000, 5000, Decimal::bps(100));
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #10)")]
    fn test_get_deposit_amounts_below_min_a() {
        let env = Env::default();
        // The calculated deposit for asset A is below the minimum requirement
        utils::get_deposit_amounts(
            &env,
            5000,
            Some(2000),
            200,
            None,
            1000,
            500,
            Decimal::bps(1000),
        );
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #12)")]
    fn test_get_deposit_amounts_below_min_b() {
        let env = Env::default();
        // The calculated deposit for asset B is below the minimum requirement
        utils::get_deposit_amounts(
            &env,
            200,
            None,
            5000,
            Some(2000),
            500,
            1000,
            Decimal::bps(120000),
        );
    }

    #[test]
    fn test_get_deposit_amounts_accept_a_within_1_percent() {
        let env = Env::default();
        // Set up the inputs so that amount_a = (1010 * 1000 / 1000) = 1010, which is > desired_a (1000),
        // but the ratio is exactly 1.01, which is within the 1% tolerance
        let result =
            utils::get_deposit_amounts(&env, 1000, None, 1010, None, 1000, 1000, Decimal::bps(100));
        assert_eq!(result, (1000, 1000));
    }

    #[test]
    fn test_get_deposit_amounts_accept_b_within_1_percent() {
        let env = Env::default();
        let result =
            utils::get_deposit_amounts(&env, 1010, None, 1000, None, 1000, 1000, Decimal::bps(100));
        assert_eq!(result, (1000, 1000));
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
    #[should_panic(expected = "Pool: Validate fee bps: Total fees cannot be greater than 100%")]
    fn test_invalidate_fee_bps() {
        let env = Env::default();
        validate_fee_bps(&env, 10_001);
    }
}
