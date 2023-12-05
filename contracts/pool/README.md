

# POOL

## Main functionality
```This is one of the DEX's core contracts. It's main purpose is to facilitate the provision and withdrawal of liquidity, swapping assets and simulating assets swap, all this by using the XYK model.```





## Main methods:
#### fn initialize(env: Env, admin: Address, share_token_decimals: u32, swap_fee_bps: i64, fee_recipient: Address, max_allowed_slippage_bps: i64, max_allowed_spread_bps: i64, max_referral_bps: i64, token_init_info: TokenInitInfo, stake_contract_info: StakeInitInfo);

**params:**
```
-- admin: Address of the contract administrator to be.
-- share_token_decimals: the number of decimals to be used for the given contract.
-- swap_fee_bps: the comission fee for the network in the given liquidity pool.
-- fee_recipient: the address that will receive the aforementioned fee.
-- max_allowed_slippage_bps: the maximum allowed slippage for a swap, set in BPS.
-- max_allowed_spread_bps: the maximum allowed difference between the price at the current moment and the price on which the users agree to sell. Measured in BPS.
-- max_referral_bps: maximum allowed referral commission measured in BPS.
-- token_init_info: Struct containing information for the initialization of one of the two tokens in the pool.
-- stake_contract_info: Struct containing information for the initialization of the stake contract for the given liquidity pool.
```
**return type:**
`void`

**description:**
`Used for the initialization of the liquidity pool contract - this sets the admin in Config, initializes both token contracts, that will be in the pool and also initializes the staking contract needed for providing liquidity.`

<hr>

#### fn provide_liquidity(env: Env, depositor: Address, desired_a: Option<i128>, min_a: Option<i128>, desired_b: Option<i128>, min_b: Option<i128>, custom_slippage_bps: Option<i64>);

**params:**
```
-- depositor: the address of the ledger calling the current method and providing liqudity for the pool
-- desired_a: Optional. Amount of the first asset that the depositor wants to provide in the pool.
-- min_a: Optional. Minimum amount of the first asset that the depositor wants to provide in the pool.
-- desired_b: Optional. Amount of the second asset that the depositor wants to provide in the pool.
-- min_b: Optional. Minimum amount of the second asset that the depositor wants to provide in the pool.
-- custom_slippage_bps: Optional amount measured in BPS for the slippage tolerance.
```
**return type:**
`void`

**description:**
`Allows the users to deposit optional pairs of tokens in the pool and receive awards in return. The awards are calculated based on the amount of assets deposited in the pool.`

<hr>

#### fn swap(env: Env, Address, referral: Option<Referral>, offer_asset: Address, offer_amount: i128, belief_price: Option<i64>, max_spread_bps: Option<i64>) -> i128;

**params:**
```
-- referral: Optional value for a Struct 'Referral' for the ledger that will receive commission from this swap. 'Referral' contains from an address of the referral and its commission fee.
-- offer_asset: Address for the asset the user wants to swap.
-- offer_amount: amount that the user wants to swap.
-- belief_price: Optional value that represents that users belived/expected price per token.
-- max_spread_bps: Optional. Maximum allowed spread/slippage for the swap.

```
**return type:**
`i128`

**description:**
`Changes one asset for another in the pool.`

<hr>

#### fn withdraw_liquidity(env: Env, recipient: Address, share_amount: i128, min_a: i128, min_b: i128) -> (i128, i128);

**params:**
```
-- recipient: Address that will receive the withdrawn liquidity.
-- share_amount: amount of shares that the user will remove from the liquidity pool.
-- min_a: amount of the first token.
-- min_b: amount of the second token.

```
**return type:**
`(i128, i128)` Tuple of the amount of the first and second token to be sent back to the user.

**description:**
`Allows for users to withdraw their liquidity out of a pool, forcing them to burn their share tokens in the given pool, before they can get the assets back.`

<hr>

#### fn update_config(env: Env, sender: Address, new_admin: Option<Address>, total_fee_bps: Option<i64>, fee_recipient: Option<Address>, max_allowed_slippage_bps: Option<i64>, max_allowed_spread_bps: Option<i64>);

**params:**
```
-- todo

```
**return type:**
`todo` 

**description:**
`todo.`

<hr>

