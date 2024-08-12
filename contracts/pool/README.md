# Dex Pool

## Main functionality
This is one of the DEX's core contracts. It's main purpose is to facilitate the provision and withdrawal of liquidity, swapping assets and simulating assets swap, all this by using the XYK model.

## Messages:
`initialize`

Params:
- `stake_wasm_hash`: `BytesN<32>` WASM file of the stake contract.
- `token_wasm_hash`: `BytesN<32>` WASM file of the token contract.
- `stake_rewards_wasm_hash`: `BytesN<32>` WASM file of the stake rewards contract.
- `lp_init_info`: `LiquidityPoolInitInfo` helper struct containing the initial information required to initialize the pool.
- `factory_addr`: `Address` of the factory used to create this pool.
- `share_token_decimals`: `u32` number of decimals to be used for the share token.
- `share_token_name`: `String` for the name of the share token.
- `share_token_symbol`: `String` ticker for the share token.
- `default_slippage_bps`: `i64` slippage to be used when the user hasn't specified any during liquidity providing.
- `max_allowed_fee_bps`: `i64` the maximum allowed fee that the contract can charge users per swap.

Return type:
void

Description:
Used for the initialization of the liquidity pool contract - this sets the admin in Config, initializes both token contracts, that will be in the pool and also initializes the staking contract needed for providing liquidity.

<hr>

`provide_liquidity`

Params:
- `sender`: `Address` of the ledger calling the current method and providing liqudity for the pool
- `desired_a`: Optional `i128` value for amount of the first asset that the depositor wants to provide in the pool.
- `min_a`: Optional `i128` value for minimum amount of the first asset that the depositor wants to provide in the pool.
- `desired_b`: Optional `i128` value for amount of the second asset that the depositor wants to provide in the pool.
- `min_b`: Optional `i128` value for minimum amount of the second asset that the depositor wants to provide in the pool.
- `custom_slippage_bps`: Optional `i64` value for amount measured in BPS for the slippage tolerance.
- `deadline`: `Option<u64>` sets a desired timestamp by which the tx should be valid. After that deadline the tx is discarded.

Return type:
void

Description:
Allows the users to deposit optional pairs of tokens in the pool and receive awards in return. The awards are calculated based on the amount of assets deposited in the pool.

<hr>

`swap`

Params:
- `sender`: `Address` of the user that requests the swap.
- `offer_asset`: `Address` for the asset the user wants to swap.
- `offer_amount`: `i128` amount that the user wants to swap.
- `ask_asset_min_amount`: `Option<i128>` value that represents the minimum amount of the ask token that the user should receive.
- `max_spread_bps`: `Option<i64>` maximum allowed spread for the swap.
- `deadline`: `Option<u64>` sets a desired timestamp by which the tx should be valid. After that deadline the tx is discarded.
- `max_allowed_fee_bps`: `Option<i64>` the maximum fee for which the user agreed to make a swap in comparison to the contract fee.

Return type:
i128

Description:
Changes one asset for another in the pool.

<hr>

`withdraw_liquidity`

Params:
- `recipient`: `Address` that will receive the withdrawn liquidity.
- `share_amount`: `i128` amount of shares that the user will remove from the liquidity pool.
- `min_a`: `i128` amount of the first token.
- `min_b`: `i128` amount of the second token.
- `deadline`: `Option<u64>` sets a desired timestamp by which the tx should be valid. After that deadline the tx is discarded.

Return type:
(i128, i128) tuple of the amount of the first and second token to be sent back to the user.

Description:
Allows for users to withdraw their liquidity out of a pool, forcing them to burn their share tokens in the given pool, before they can get the assets back.

<hr>

`update_config`

Params:
- `new_admin`: `Option<Address>` of the new admin for liquidity pool
- `total_fee_bps`: `Option<i64>` value for the total fees (in bps) charged by the pool
- `fee_recipient`: `Option<Address>` for the recipient of the swap commission fee
- `max_allowed_slippage_bps`: `Option<i64>` value the maximum allowed slippage for a swap, set in BPS.
- `max_allowed_spread_bps`: `Option<i64>` value for maximum allowed difference between the price at the current moment and the price on which the users agree to sell. Measured in BPS.
- `max_referral_bps`: `Option<i64>` value for the maximum referral fee, measured in bps. 

Return type:
void

Description:
Updates the liquidity pool `Config` information with new one.

<hr>

`upgrade`

Params:
- `new_wasm_hash`: `WASM hash` of the new liquidity pool contract

Return type:
void

Description:
Migration entrypoint

<hr>

## Queries:
`query_config`

Params:
`None`

Return type:
`Config` struct.

Description:
Queries the contract `Config` 

<hr>

`query_share_token_address`

Params:
`None`

Return type:`
`Address` of the pool's share token.

Description:
Returns the address for the pool share token.

<hr>

`query_stake_contract_address`

Params:
`None`

Return type:
`Address` of the pool's stake contract.

Description:
Returns the address for the pool stake contract. 

<hr>

`query_pool_info`

Params
`None`

Return type:
`PoolResponse` struct represented by two token assets and share token.

Description:
Returns  the total amount of LP tokens and assets in a specific pool. 

<hr>

`query_pool_info_for_factory`

Params:
`None`

Return type:
`LiquidityPoolInfo` struct representing information relevant for the liquidity pool.

Description:
Returns all the required information for a liquidity pool that is called by the factory contract. 

<hr>


`simulate_swap`

Params:
- `offer_asset`: `Address` of the token that the user wants to sell.
- `sell_amount`: `i128` value for the total amount that the user wants to sell.

Return type:
`SimulateSwapResponse` struct represented by `ask_amount: i128`, `commission_amount: i128`, `spread_amount: i128` and `total_return: i128`.

Description:
Simulate swap transaction. 
<hr>

`simulate_reverse_swap`

Params:
- `ask_asset`: `Address` of the token that the user wants to buy.
- `ask_amount`: `i128` value for the total amount that the user wants to buy.

Return type:
`SimulateReverseSwapResponse` struct represented by `offer_amount: i128`, `commission_amount: i128` and `spread_amount: i128`.

Description:
Simulate reverse swap transaction. 
<hr>

`query_share`

Params:
- `amount`: `i128` amount for which we will find the assets info in share contract.

Return type:
`(Asset, Asset)` tuple with structs providing information about the share contract.

Description:
Helper that provides information about the assets used for a given amount of shared tokens.
<hr>

`query_total_issued_lp`

Params:
- `None`

Return type:
`i128` total number of the shares issued by the current contract.

Description:
Helper function that keeps track of the total lp tokens issued by the contract. This number changes accordingly when the users provide or withdraw liquidity.
<hr>

## Internal Structs

```rs
pub struct TokenInitInfo {
    pub token_a: Address,
    pub token_b: Address,
}
```


```rs
pub struct StakeInitInfo {
    pub min_bond: i128,
    pub min_reward: i128,
    pub manager: Address,
    pub max_complexity: u32,
}
```

```rs
pub struct LiquidityPoolInitInfo {
    pub admin: Address,
    pub swap_fee_bps: i64,
    pub fee_recipient: Address,
    pub max_allowed_slippage_bps: i64,
    pub default_slippage_bps: i64,
    pub max_allowed_spread_bps: i64,
    pub max_referral_bps: i64,
    pub token_init_info: TokenInitInfo,
    pub stake_init_info: StakeInitInfo,
}
```

```rs
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
```

```rs
pub struct LiquidityPoolInfo {
    pub pool_address: Address,
    pub pool_response: PoolResponse,
    pub total_fee_bps: i64,
}
```

```rs
pub struct SimulateSwapResponse {
    pub ask_amount: i128,
    pub commission_amount: i128,
    pub spread_amount: i128,
    pub total_return: i128,
}
```

```rs
pub struct SimulateReverseSwapResponse {
    pub offer_amount: i128,
    pub commission_amount: i128,
    pub spread_amount: i128,
}
```

