# MIGRATION

This file shows the API changes between different versions of Phoenix-Contracts.

## 1.0.0 -> 1.1.0

### Factory contract

#### Summary
Factory allows for the creation of stable pools. This comes with the following changes:

Messages 
* `initialize` function now accepts new argumen for the stable pool `wasm_hash`
The new signature is as follows
```Git
    fn initialize(
        env: Env,
        admin: Address,
        multihop_wasm_hash: BytesN<32>,
        lp_wasm_hash: BytesN<32>,
     ++ stable_wasm_hash: BytesN<32>,
        stake_wasm_hash: BytesN<32>,
        token_wasm_hash: BytesN<32>,
        whitelisted_accounts: Vec<Address>,
        lp_token_decimals: u32,
    );
```

* `create_liquidity_pool` now accepts two new arguments `pool_type` and `amp`.

  `pool_type` is an Enum with two variatns - `Xyk` and `Stable`.

  `amp` is an Option<u64> which is only required when `pool_type` is `Stable`.

  The new signature is as follows:
  ```git
      fn create_liquidity_pool(
        env: Env,
        sender: Address,
        lp_init_info: LiquidityPoolInitInfo,
        share_token_name: String,
        share_token_symbol: String,
     ++ pool_type: PoolType,
     ++ amp: Option<u64>,
    ) -> Address;
  ```