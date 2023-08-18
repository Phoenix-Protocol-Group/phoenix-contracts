use decimal::Decimal;
use soroban_sdk::{contracttype, Address, BytesN};

// Validate if int value is bigger then 0
#[macro_export]
macro_rules! validate_int_parameters {
    ($($arg:expr),*) => {
        {
            let mut res: Result<(), $crate::error::ContractError> = Ok(());
            $(
                let value: Option<i128> = Into::<Option<_>>::into($arg);
                if let Some(val) = value {
                    if val <= 0 {
                        res = Err($crate::error::ContractError::ArgumentsInvalidLessOrEqualZero);
                    }
                }
            )*
            res
        }
    };
}

pub fn assert_approx_ratio(a: Decimal, b: Decimal, tolerance: Decimal) -> bool {
    let diff = (a - b).abs();
    diff <= tolerance
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenInitInfo {
    pub token_wasm_hash: BytesN<32>,
    pub token_a: Address,
    pub token_b: Address,
}

impl TokenInitInfo {
    pub fn get_token_wasm_hash(&self) -> BytesN<32> {
        self.token_wasm_hash.clone()
    }

    pub fn get_token_a(&self) -> Address {
        self.token_a.clone()
    }

    pub fn get_token_b(&self) -> Address {
        self.token_b.clone()
    }
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StakeInitInfo {
    pub stake_wasm_hash: BytesN<32>,
    pub token_per_power: u128,
    pub min_bond: i128,
    pub max_distributions: u32,
    pub min_reward: i128,
}

impl StakeInitInfo {
    pub fn get_stake_wasm_hash(&self) -> BytesN<32> {
        self.stake_wasm_hash.clone()
    }

    pub fn get_token_per_power(&self) -> u128 {
        self.token_per_power
    }

    pub fn get_min_bond(&self) -> i128 {
        self.min_bond
    }

    pub fn get_max_distributions(&self) -> u32 {
        self.max_distributions
    }

    pub fn get_min_reward(&self) -> i128 {
        self.min_reward
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_int_parameters() {
        // The macro should not panic for valid parameters.
        validate_int_parameters!(1, 2, 3).unwrap();
        validate_int_parameters!(1, 1, 1).unwrap();
        validate_int_parameters!(1i128, 2i128, 3i128, Some(4i128), None::<i128>).unwrap();
        validate_int_parameters!(None::<i128>, None::<i128>).unwrap();
        validate_int_parameters!(Some(1i128), None::<i128>).unwrap();

        validate_int_parameters!(1, -2, 3).unwrap_err();
        validate_int_parameters!(0, 1, 3).unwrap_err();
        validate_int_parameters!(1, 1, 0).unwrap_err();
        validate_int_parameters!(Some(0i128), None::<i128>).unwrap_err();
        validate_int_parameters!(Some(-1i128), None::<i128>).unwrap_err();
    }

    #[test]
    fn test_assert_approx_ratio_close_values() {
        let a = Decimal::from_ratio(100, 101);
        let b = Decimal::from_ratio(100, 100);
        let tolerance = Decimal::percent(3);
        assert!(assert_approx_ratio(a, b, tolerance));
    }

    #[test]
    fn test_assert_approx_ratio_equal_values() {
        let a = Decimal::from_ratio(100, 100);
        let b = Decimal::from_ratio(100, 100);
        let tolerance = Decimal::percent(3);
        assert!(assert_approx_ratio(a, b, tolerance));
    }

    #[test]
    fn test_assert_approx_ratio_outside_tolerance() {
        let a = Decimal::from_ratio(100, 104);
        let b = Decimal::from_ratio(100, 100);
        let tolerance = Decimal::percent(3);
        assert!(!assert_approx_ratio(a, b, tolerance));
    }
}
