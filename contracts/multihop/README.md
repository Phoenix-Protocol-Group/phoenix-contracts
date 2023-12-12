# Dex Multihop

## Main functionality
The main purpose of the multihop contract is to provide the ability of the users to swap tokens between multiple liquidity pools.



## Messages:
`initialize`

Params:

- `admin`: `Address` of the contract administrator to be
- `factory`: `Address` of the factory contract to be deployed initially

Return type:
void

Description:
Used for the initialization of the multihop contract - this sets the multihop contract as initialized, stores the admin and factory address in the Config struct

<hr>

`swap`

Params:

- `recipient`: `Address` of the contract that will receive the amount swapped.
- `referral`: `Option<Address>` of the referral, that will get a referral commission bonus for the swap.
- `operations`: `Vec<Swap>` that holds both the addresses of the asked and offer assets.
- `max_belief_price`: `Option<i64>` value for the maximum believe price that will be used for the swaps.
- `max_spread_bps`: `Option<i64>` maximum permitted difference between the asked and offered price in BPS.
- `amount`: `i128` value representing the amount offered for swap

Return type:
void

Description:
Takes a list of `Swap` operations between the different pools and iterates over them, swapping the tokens in question by calling the pool contract.

<hr>

`simulate_swap`
Params:

- `operations`: `Vec<Swap>`holding the addresses of the asked and offer assets
- `amount`: `i128` value representing the amount that should be swapped

Return type:
`SimulateSwapResponse` containing the details of the swap

Description:
Dry runs a swap operation. This is useful when we want to display some additional information such as pool commission fee, slippage tolerance and expected returned values from the swap in question.

<hr>

`simulate_reverse_swap`

Params:

- `operations`: `Vec<Swap>` holding the addresses of the asked and offer assets
- `amount`: `i128` value representing the amount that should be swapped

Return type:
`SimulateReverseSwapResponse` containing the details of the same swap but in reverse

Description:
Dry runs a swap operation but in reverse. This is useful when we want to display some additional information such as pool commission fee, slippage tolerance and expected returned values from the reversed swap in question.

<hr>

`get_admin`
Params:

* None

Return type:
`Address` of the admin for the current Multihop contract.

Description:
Queries for the admin address of the current multihop contract.
