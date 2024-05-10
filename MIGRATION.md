# MIGRATION

This file shows the API changes between different versions of Phoenix-Contracts.

## 1.0.0 -> 1.1.0

### Factory contract

#### Summary
Factory allows for the creation of stable pools. This comes with the following changes:

Messages 
* `initialize` function now accepts new argument `stable_wasm_hash: BytesN<32>` as the 4th element.

* `create_liquidity_pool` now accepts two new arguments `pool_type` and `amp`.

  `pool_type` is an Enum with two variatns - `Xyk` and `Stable` and is 5th element.

  `amp` is an Option<u64> which is only required when `pool_type` is `Stable`. Added as 6th element.
