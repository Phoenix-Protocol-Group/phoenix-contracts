# Dex Decimal
This code is taken from the [cosmwasm-std crate](https://github.com/CosmWasm/cosmwasm.), which is licensed under the Apache License 2.0
The contract provides a `Decimal` struct for arithmetic operations, suitable for blockchain De-Fi operations, where precision is of highest importance. It ensures that calculations are accurate up to 18 decimal places.

## Messages

- `new(value: i128) -> Self`: Creates a new Decimal.
- `raw(value: i128) -> Self`: Returns the raw value from `i128`.
- `one() -> Self`: Create a `1.0` Decimal.
- `zero() -> Self`: Create a `0.0` Decimal.
- `percent(x: i64) -> Self`: Convert `x%` into Decimal.
- `permille(x: i64) -> Self`: Convert permille `(x/1000)` into Decimal.
- `bps(x: i64) -> Self`: Convert basis points `(x/10000)` into Decimal.
- `from_atomics(atomics: i128, decimal_places: i32) -> Self`: Creates a Decimal from atomic units and decimal places.
- `inv(&self) -> Option<Self>`: Returns the multiplicative inverse `1/d` for decimal `d`.
- `from_ratio(numerator: impl Into<i128>, denominator: impl Into<i128>) -> Self`: Returns the ratio (numerator / denominator) as a Decimal.
- `abs(&self) -> Self`: Returns the absolute value of the Decimal.
- `to_string(&self, env: &Env) -> String`: Converts the Decimal to a string.
