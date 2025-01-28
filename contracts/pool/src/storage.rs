use phoenix::ttl::{PERSISTENT_BUMP_AMOUNT, PERSISTENT_LIFETIME_THRESHOLD};
use soroban_sdk::{
    contracttype, log, panic_with_error, symbol_short, xdr::ToXdr, Address, Bytes, BytesN,
    ConversionError, Env, Symbol, TryFromVal, Val,
};

use crate::{error::ContractError, token_contract};
use soroban_decimal::Decimal;

pub const ADMIN: Symbol = symbol_short!("ADMIN");

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

const DEFAULT_SLIPPAGE_BPS: Symbol = symbol_short!("DSLIPBPS");
pub fn save_default_slippage_bps(env: &Env, bps: i64) {
    env.storage().persistent().set(&DEFAULT_SLIPPAGE_BPS, &bps);
    env.storage().persistent().extend_ttl(
        &DEFAULT_SLIPPAGE_BPS,
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_BUMP_AMOUNT,
    )
}

pub fn get_default_slippage_bps(env: &Env) -> i64 {
    let bps = env
        .storage()
        .persistent()
        .get(&DEFAULT_SLIPPAGE_BPS)
        .expect("Stable wasm hash not set");

    env.storage().persistent().extend_ttl(
        &DEFAULT_SLIPPAGE_BPS,
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_BUMP_AMOUNT,
    );
    bps
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
    let config = env.storage().persistent().get(&CONFIG).unwrap();
    env.storage().persistent().extend_ttl(
        &CONFIG,
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_BUMP_AMOUNT,
    );
    config
}

pub fn save_config(env: &Env, config: Config) {
    env.storage().persistent().set(&CONFIG, &config);
    env.storage().persistent().extend_ttl(
        &CONFIG,
        PERSISTENT_LIFETIME_THRESHOLD,
        PERSISTENT_BUMP_AMOUNT,
    );
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
    /// The address of the Stake contract for the liquidity pool
    pub stake_address: Address,
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
    use phoenix::ttl::{INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD};
    use soroban_sdk::String;

    use super::*;

    #[allow(clippy::too_many_arguments)]
    pub fn deploy_token_contract(
        env: &Env,
        token_wasm_hash: BytesN<32>,
        token_a: &Address,
        token_b: &Address,
        admin: Address,
        decimals: u32,
        name: String,
        symbol: String,
    ) -> Address {
        let mut salt = Bytes::new(env);
        salt.append(&token_a.clone().to_xdr(env));
        salt.append(&token_b.clone().to_xdr(env));
        let salt = env.crypto().sha256(&salt);
        env.deployer()
            .with_current_contract(salt)
            .deploy_v2(token_wasm_hash, (admin, decimals, name, symbol))
    }

    pub fn deploy_stake_contract(e: &Env, stake_wasm_hash: BytesN<32>) -> Address {
        let salt = Bytes::new(e);
        let salt = e.crypto().sha256(&salt);

        e.deployer()
            .with_current_contract(salt)
            .deploy_v2(stake_wasm_hash, ())
    }

    pub fn save_admin_old(e: &Env, address: Address) {
        e.storage().persistent().set(&DataKey::Admin, &address);
        e.storage().persistent().extend_ttl(
            &DataKey::Admin,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    pub fn _save_admin(e: &Env, address: Address) {
        e.storage().instance().set(&ADMIN, &address);
        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
    }

    pub fn save_total_shares(e: &Env, amount: i128) {
        e.storage().persistent().set(&DataKey::TotalShares, &amount);
        e.storage().persistent().extend_ttl(
            &DataKey::TotalShares,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    pub fn save_pool_balance_a(e: &Env, amount: i128) {
        e.storage().persistent().set(&DataKey::ReserveA, &amount);
        e.storage().persistent().extend_ttl(
            &DataKey::ReserveA,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }

    pub fn save_pool_balance_b(e: &Env, amount: i128) {
        e.storage().persistent().set(&DataKey::ReserveB, &amount);
        e.storage().persistent().extend_ttl(
            &DataKey::ReserveB,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
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
        let admin = e.storage().persistent().get(&DataKey::Admin).unwrap();
        e.storage().persistent().extend_ttl(
            &DataKey::Admin,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        admin
    }

    pub fn _get_admin(e: &Env) -> Address {
        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        e.storage().instance().get(&ADMIN).unwrap_or_else(|| {
            log!(e, "XYZ Pool: Admin not set");
            panic_with_error!(&e, ContractError::AdminNotSet)
        })
    }

    pub fn get_total_shares(e: &Env) -> i128 {
        let total_shares = e.storage().persistent().get(&DataKey::TotalShares).unwrap();
        e.storage().persistent().extend_ttl(
            &DataKey::TotalShares,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        total_shares
    }
    pub fn get_pool_balance_a(e: &Env) -> i128 {
        let balance_a = e.storage().persistent().get(&DataKey::ReserveA).unwrap();
        e.storage().persistent().extend_ttl(
            &DataKey::ReserveA,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        balance_a
    }

    pub fn get_pool_balance_b(e: &Env) -> i128 {
        let balance_b = e.storage().persistent().get(&DataKey::ReserveB).unwrap();
        e.storage().persistent().extend_ttl(
            &DataKey::ReserveB,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );

        balance_b
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
        if desired_a <= 0 || desired_b <= 0 {
            log!(
            env,
            "Pool: Get Deposit Amounts: Desired amounts are equal or less than zero - desired_a: {}, desired_b: {}",
            desired_a, desired_b);

            panic_with_error!(env, ContractError::DesiredAmountsBelowOrEqualZero);
        }

        if let (Some(min_a), Some(min_b)) = (min_a, min_b) {
            if min_a < 0 || min_b < 0 {
                log!(
                env,
                "Pool: Get Deposit Amounts: Min amounts are less than zero - min_a: {}, min_b: {}",
                min_a, min_b);

                panic_with_error!(env, ContractError::MinAmountsBelowZero);
            }
        }

        if pool_balance_a == 0 && pool_balance_b == 0 {
            return (desired_a, desired_b);
        }

        if let Some(min_a) = min_a {
            if min_a > desired_a {
                log!(
                    &env,
                    "Pool: GetDepositAmounts: Critical error - minimumA is bigger than desiredA"
                );
                panic_with_error!(env, ContractError::GetDepositAmountsMinABiggerThenDesiredA);
            }
        }
        if let Some(min_b) = min_b {
            if min_b > desired_b {
                log!(
                    &env,
                    "Pool: GetDepositAmounts: Critical error - minimumB is bigger than desiredB"
                );
                panic_with_error!(env, ContractError::GetDepositAmountsMinBBiggerThenDesiredB);
            }
        }

        let amount_a = {
            let mut amount_a = desired_b
                .checked_mul(pool_balance_a)
                .and_then(|result| result.checked_div(pool_balance_b))
                .unwrap_or_else(|| {
                    log!(&env, "Pool: Get Deposit Amounts: overflow/underflow error");
                    panic_with_error!(env, ContractError::ContractMathError);
                });
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
            let mut amount_b = desired_a
                .checked_mul(pool_balance_b)
                .and_then(|result| result.checked_div(pool_balance_a))
                .unwrap_or_else(|| {
                    log!(&env, "Pool: Get Deposit Amounts: overflow/underflow error");
                    panic_with_error!(env, ContractError::ContractMathError);
                });
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
        e.storage().persistent().extend_ttl(
            &DataKey::Initialized,
            PERSISTENT_LIFETIME_THRESHOLD,
            PERSISTENT_BUMP_AMOUNT,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use test_case::test_case;

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
    fn test_get_deposit_amounts_pool_balances_zero() {
        let env = Env::default();
        let result =
            utils::get_deposit_amounts(&env, 100, Some(50), 200, Some(50), 0, 0, Decimal::bps(100));
        assert_eq!(result, (100, 200));
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #211)")]
    fn test_get_deposit_amounts_amount_b_less_than_desired() {
        let env = Env::default();
        utils::get_deposit_amounts(&env, 1000, None, 1005, Some(1001), 1, 1, Decimal::bps(100));
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #210)")]
    fn test_get_deposit_amounts_amount_b_exceeds_desired_amount() {
        let env = Env::default();
        utils::get_deposit_amounts(
            &env,
            1100,
            Some(1000),
            1000,
            Some(1000),
            1,
            1,
            Decimal::bps(100),
        );
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #211)")]
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
    #[should_panic(expected = "Error(Contract, #206)")]
    fn test_get_deposit_amounts_amount_a_greater_than_desired_and_less_than_min_a() {
        let env = Env::default();
        utils::get_deposit_amounts(&env, 50, Some(100), 200, None, 100, 200, Decimal::bps(100));
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #207)")]
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
    #[should_panic(expected = "Error(Contract, #206)")]
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
    #[should_panic(expected = "Error(Contract, #208)")]
    fn test_get_deposit_amounts_exceeds_desired() {
        let env = Env::default();
        // The calculated deposit for asset A exceeds the desired amount and is not within 1% tolerance
        utils::get_deposit_amounts(&env, 1000, None, 2000, None, 10000, 5000, Decimal::bps(100));
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #209)")]
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
    #[should_panic(expected = "Error(Contract, #211)")]
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

    #[test_case(-1, 10 ; "when desired_a is negative")]
    #[test_case(0, 10 ; "when desired_a is zero")]
    #[test_case(10, -1 ; "when desired_b is negative")]
    #[test_case(10, 0 ; "when desired_b is zero")]
    #[test_case(-1, -1 ; "when both desired are negative")]
    #[test_case(0, 0 ; "when both desired are zero")]
    #[should_panic(expected = "Error(Contract, #213)")]
    fn test_get_deposit_amounts_desired_less_than_or_equal_zero(desired_a: i128, desired_b: i128) {
        let env = Env::default();
        utils::get_deposit_amounts(
            &env,
            desired_a,
            Some(100),
            desired_b,
            Some(300),
            100,
            200,
            Decimal::bps(100),
        );
    }

    #[test_case(-1, 10 ; "when min_a is negative")]
    #[test_case(10, -1 ; "when min_b is negative")]
    #[test_case(-1, -1 ; "when both minimums are negative")]
    #[should_panic(expected = "Error(Contract, #214)")]
    fn test_get_deposit_amounts_min_amounts_less_than_zero(min_a: i128, min_b: i128) {
        let env = Env::default();
        utils::get_deposit_amounts(
            &env,
            100,
            Some(min_a),
            100,
            Some(min_b),
            100,
            200,
            Decimal::bps(100),
        );
    }

    #[test]
    fn test_max_allowed_slippage() {
        let env = Env::default();
        let config = Config {
            max_allowed_slippage_bps: 100,
            token_a: Address::generate(&env),
            token_b: Address::generate(&env),
            share_token: Address::generate(&env),
            stake_contract: Address::generate(&env),
            pool_type: PairType::Xyk,
            total_fee_bps: 10i64,
            fee_recipient: Address::generate(&env),
            max_allowed_spread_bps: 10_i64,
            max_referral_bps: 10i64,
        };

        let result = config.max_allowed_slippage();

        assert_eq!(
            result,
            Decimal::percent(1),
            "Max allowed slippage should be 1%."
        );
    }
}
