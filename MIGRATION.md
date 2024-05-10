# MIGRATION

This file shows the API changes between different versions of Phoenix-Contracts.

## 1.0.0 -> X.X.X

### factory

* `initialize` function requires a new argument `stable_wasm_hash: BytesN<32>`

* `create_liquidity_pool` requires a new argument `pool_type` parameter (and optional `amp` for stable pool)
