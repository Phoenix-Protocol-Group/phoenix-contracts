#[macro_export]
macro_rules! validate_int_parameters {
    ($($x:expr),+ $(,)?) => {
        (|| {
            $(
                let num = $x;
                if num <= 0 {
                    return Err($crate::error::ContractError::ArgumentsInvalidLessOrEqualZero);
                }
            )+
            Ok(())
        })()
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_validate_int_parameters() {
        // The macro should not panic for valid parameters.
        validate_int_parameters!(1, 2, 3).unwrap();
        validate_int_parameters!(1, 1, 1).unwrap();
        validate_int_parameters!(1, -2, 3).unwrap_err();
        validate_int_parameters!(0, 1, 3).unwrap_err();
        validate_int_parameters!(1, 1, 0).unwrap_err();
    }
}
