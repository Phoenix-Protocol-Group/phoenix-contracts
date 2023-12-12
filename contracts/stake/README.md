# STAKING

## Main functionality
Provides staking capabilities, reward distribution and reward management functionalities to the Phoenix DEX.

## Messages:
`initialize`

Params:
- `admin`: `Address` of the administrator for the contract
- `lp_token`: `Address` of the liquidity pool used with this stake contract
- `min_bond`: `i128` value showing the minimum required bond
- `max_distributions`: `u32` value showing the maximum number of distributions
- `min_reward`: `i128` the minimum amount of rewards the user can withdraw.

Return type:
void

Description:
Used to set up the staking contract with the initial parameters.

<hr>

`bond`

Params:
- `sender`: `Address` of the user that sends tokens to the stake contract.
- `tokens`: `i128` value representing the number of tokens the user sends.

Return type:
void

Description:
Allows for users to stake/bond their lp tokens

<hr>

`unbond`

Params:
- `sender`: `Address` of the user that wants to unbond/unstake their tokens.
- `stake_amount`: `i128` value representing the numbers of stake to be unbond.
- `take_timestamp`: `u64`value used to calculate the correct stake to be removed

Return type:
void

Description:
Allows the user remove their staked tokens from the stake contract, with any rewards they may have earned, based on the amount of staked tokens and stake's timestamp.

<hr>

`create_distribution_flow`

Params:
- `sender`: `Address` of the user that creates the flow
- `manager`: `Address` of the user that will be managing the flow
- `asset`: `Address` of the asset that will be used in the distribution flow

Return type:
void

Description:
Creates a distribution flow for sending rewards, that are managed by a  manager for a specific asset.

<hr>

`distribute_rewards`

Params:
None

Return type:
void

Description:
Sends the rewards to all the users that have stakes, on the basis of the current reward distribution rule set and total staked amount.

<hr>

`withdraw_rewards`

Params:
- `sender`: `Address` of the user that wants to withdraw their rewards

Return type:
void

Description:
Allows for users to withdraw their rewards from the stake contract.

<hr>

`fund_distribution`

Params:
- `sender`: `Address` of the user that calls this method.
- `start_time`: `u64` value representing the time in which the funding has started.
- `distribution_duration`: `u64` value representing the duration for the distribution in seconds
- `token_address`: `Address` of the token that will be used for the reward distribution 
- `token_amount`: `i128` value representing how many tokens will be allocated for the distribution time


Return type:
void

Description:
Sends funds for a reward distribution.

<hr>

## Queries:
`query_config`

Params:
None

Return type:
`ConfigResponse` struct.

Description:
Queries the contract `Config` 

<hr>

`query_admin`

Params:
None

Return type:
`Address` struct.

Description:
Returns the address of the admin for the given stake contract. 

<hr>

`query_staked`

Params:
- `address`: `Address` of the stake contract we want to query

Return type:
`StakedResponse` struct.

Description:
Provides information about the stakes of a specific address. 

<hr>

`query_total_staked`

Params:
None

Return type:
`i128` 

Description:
Returns the total amount of tokens currently staked in the contract. 

<hr>

`query_annualized_rewards`

Params:
None

Return type:
`AnnualizedRewardsResponse`  struct

Description:
Provides an overview of the annualized rewards for each distributed asset. 

<hr>

`query_withdrawable_rewards`

Params:
- `address`: `Address` whose rewards we are searching

Return type:
`WithdrawableRewardsResponse`  struct

Description:
Queries the amount of rewards that a given address can withdraw. 

<hr>

`query_distributed_rewards`

Params:
- `asset`: `Address` of the token for which we query

Return type:
`u128`

Description:
Reports the total amount of rewards distributed for a specific asset.

<hr>

`query_undistributed_rewards`

Params:
- `asset`: `Address` of the token for which we query

Return type:
`u128`

Description:
Queries the total amount of remaining rewards for a given asset.
