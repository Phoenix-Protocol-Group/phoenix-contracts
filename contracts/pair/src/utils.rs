#[macro_export]
macro_rules! validate_int_parameters {
    ($($arg:expr),*) => {
        {
            let mut res: Result<(), $crate::error::ContractError> = Ok(());
            $(
                let value: Option<i128> = Into::<Option<i128>>::into($arg);
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

#[cfg(test)]
mod tests {
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
}
