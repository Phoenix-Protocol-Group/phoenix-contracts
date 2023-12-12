# Dex Pool

## Main functionality
This is one of the DEX's core contracts. It's main purpose is to facilitate the provision and withdrawal of liquidity, swapping assets and simulating assets swap, all this by using the XYK model.

## Messages:
`initialize`

Params:
- `admin`: `Address` of the contract administrator to be.
- `share_token_decimals`: `u32` value for the number of decimals to be used for the given contract.
- `swap_fee_bps`: `i64` value for the comission fee for the network in the given liquidity pool.
- `fee_recipient`: `Address` that will receive the aforementioned fee.
- `max_allowed_slippage_bps`: `i64` value for the maximum allowed slippage for a swap, set in BPS.
- `max_allowed_spread_bps`: `i64` value for the maximum allowed difference between the price at the current moment and the price on which the users agree to sell. Measured in BPS.
- `max_referral_bps`: `i64` value for maximum allowed referral commission measured in BPS.
- `token_init_info`: `TokenInitInfo` struct containing information for the initialization of one of the two tokens in the pool.
- `stake_contract_info`: `StakeInitInfo` struct containing information for the initialization of the stake contract for the given liquidity pool.

Return type:
void

Description:
Used for the initialization of the liquidity pool contract - this sets the admin in Config, initializes both token contracts, that will be in the pool and also initializes the staking contract needed for providing liquidity.

<hr>

`provide_liquidity`

Params:
- `depositor`: `Address` of the ledger calling the current method and providing liqudity for the pool
- `desired_a`: Optional `i128` value for amount of the first asset that the depositor wants to provide in the pool.
- `min_a`: Optional `i128` value for minimum amount of the first asset that the depositor wants to provide in the pool.
- `desired_b`: Optional `i128` value for amount of the second asset that the depositor wants to provide in the pool.
- `min_b`: Optional `i128` value for minimum amount of the second asset that the depositor wants to provide in the pool.
- `custom_slippage_bps`: Optional `i64` value for amount measured in BPS for the slippage tolerance.

Return type:
void

Description:
Allows the users to deposit optional pairs of tokens in the pool and receive awards in return. The awards are calculated based on the amount of assets deposited in the pool.

<hr>

`swap`

Params:
- `sender`: `Address` of the user that requests the swap.
- `referral`: Optional value for a Struct `Referral` for the ledger that will receive commission from this swap. `Referral` contains from an address of the referral and its commission fee.
- `offer_asset`: `Address` for the asset the user wants to swap.
- `offer_amount`: `i128` amount that the user wants to swap.
- `belief_price`: Optional `i64` value that represents that users belived/expected price per token.
- `max_spread_bps`: Optional `i64` value representing maximum allowed spread/slippage for the swap.

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

Return type:
(i128, i128) tuple of the amount of the first and second token to be sent back to the user.

Description:
Allows for users to withdraw their liquidity out of a pool, forcing them to burn their share tokens in the given pool, before they can get the assets back.

<hr>

`update_config`

Params:
- `sender`: `Address` of sender that wants to update the `Config`
- `new_admin`: Optional `Address` of the new admin for liquidity pool
- `total_fee_bps`: Optional `i64` value for the total fees (in bps) charged by the pool
- `fee_recipient`: Optional `Address` for the recipient of the swap commission fee
- `max_allowed_slippage_bps`: Optional `i64` value the maximum allowed slippage for a swap, set in BPS.
- `max_allowed_spread_bps`: Optional `i64` value for maximum allowed difference between the price at the current moment and the price on which the users agree to sell. Measured in BPS.

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
