

# MULTIHOP

## Main functionality
```The main purpose of the multihop contract is to provide the ability of the users to swap tokens between multiple liquidity pools.```



## Main methods:
#### fn initialize(env: Env, admin: Address, factory: Address);

**params:**
```
-- admin: Address of the contract administrator to be
-- factory: address of the factory contract to be deployed initially
```
**return type:**
`void`

**description:**
`Used for the initialization of the multihop contract - this sets the multihop contract as initialized, stores the admin and factory address in the Config struct`

<hr>

#### fn swap( env: Env, recipient: Address, referral: Option<Referral>, operations: Vec<Swap>, max_belief_price: Option<i64>, max_spread_bps: Option<i64>, amount: i128);

**params:**
```
-- recipient: Address of the contract that will receive the amount swapped.
-- referral: Optional address of the referral, that will get a referral commission bonus for the swap.
-- operations: A list of Swap struct, that holds both the addresses of the asked and offer assets.
--  max_belief_price: Optional value for the maximum believ price that will be used for the swaps.
-- max_spread_bps: maximum permitted difference between the asked and offered price in BPS.
-- amount: The amount offered for swap
```
**return type:**
`void`

**description:**
`Takes a list of 'Swap' operations between the different pools and iterates over them, swapping the tokens in question by calling the pool contract.`

<hr>

#### fn simulate_swap(env: Env, operations: Vec<Swap>, amount: i128) -> SimulateSwapResponse;

**params:**
```
-- operations: A list of `Swap` structs, each holding the addresses of the asked and offer assets
-- amount: The amount that should be swapped
```
**return type:**
`SimulateSwapResponse` containing the details of the swap

**description:**
`Dry runs a swap operation. This is useful when we want to display some additional information such as pool commission fee, slippage tolerance and expected returned values from the swap in question.`

<hr>

#### fn simulate_reverse_swap( env: Env, operations: Vec<Swap>, amount: i128) -> SimulateReverseSwapResponse;

**params:**
```
-- operations: A list of `Swap` structs, each holding the addresses of the asked and offer assets
-- amount: The amount that should be swapped
```
**return type:**
`SimulateReverseSwapResponse` containing the details of the same swap but in reverse

**description:**
`Dry runs a swap operation but in reverse. This is useful when we want to display some additional information such as pool commission fee, slippage tolerance and expected returned values from the reversed swap in question.`

<hr>

#### fn get_admin(env: Env) -> Address;
**params:**
```
-- None
```
**return type:**
`Address` of the admin for the current Multihop contract

**description:**
`Queries for the admin address of the current multihop contract`

