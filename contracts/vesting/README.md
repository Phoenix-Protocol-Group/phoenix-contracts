# Phoenix Protocol Vesting

## Main functionality
This is the vesting contract for the Phoenix Protocol. It's purpose is to regulate the distribution of tokens/assets over 
a certain time period in accordance to some predifined **conditions**.

## Messages

`initialize`

Params:
- `admin`: `Address` of the admin.
- `vesting_token`: `VestingTokenInfo` Struct representing relevant informatio to the token that will be vested.
- `vesting_balances`: `Vec<VestingBalance>` vector of structs that holds the address, balance and curve of the initial vesting balances.
- `minter_info`: `Option<MinterInfo>` address and capacity (curve) for the minter.
- `max_vesting_complexity`: `u32` maximum allowed complexity of the vesting curve.

Return type:
`Result<(), ContractError>`

Description:
Initializes the vesting contract with the given parameters.

<hr>

`transfer_token`

Params:
- `from`: `Address` of the sender.
- `to`: `Address` of the receiver.
- `amount`: `i128` amount of tokens to transfer.

Return type:
`Result<(), ContractError>`

Description:
Transfers the given amount of tokens from the sender to the receiver obeying the vesting rules.

<hr>

`burn`

Params:
- `sender`: `Address` of the sender.
- `amount`: `i128` amount of tokens to burn.

Return type:
`Result<(), ContractError>`

Description:
Burns the given amount of tokens from the sender.

<hr>

`mint`

Params:
- `sender`: `Address` of the sender.
- `to`: `Address` of the receiver.
- `amount`: `i128` amount of tokens to mint.

Return type:
Void

Description:
Mints the given amount of tokens to the receiver.

<hr>

`update_minter`

Params:
- `sender`: `Address` of the sender.
- `new_minter`: `Address` new minter address.

Return type:
Void

Description:
Updates the minter address.

<hr>

`update_minter_capacity`

Params:
- `sender`: `Address` of the sender.
- `new_capacity`: `u128` new capacity of the minter.

Return type:
Void

Description:
Updates the minter capacity by completely replacing it.

<hr>

## Queries

`query_balance`

Params:
- `address`: `Address` of the account we query

Return type:
`i128` balance of the account.

Description:
Queries the balance of the given account.

<hr>

`query_distribution_info`

Params:
- `address`: `Address` of the account we query

Return type:
`Result<DistributionInfo, ContractError>` curve of the account.

Description:
Queries the distribution info (Curve) of the given account.

<hr>

`query_token_info`

Params:
None

Return type:
`VestingTokenInfo` struct representing the token information.

Description:
Queries the token information.

<hr>

`query_minter`

Params:
None

Return type:
`MinterInfo` struct representing the minter information.

Description:
Queries the minter information.

<hr>

`query_vesting_contract_balance`

Params:
None

Return type:
`i128` total supply of the vesting token.

Description:
Queries the total supply of the vesting token.

<hr>