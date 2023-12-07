# STAKING

## Main functionality
```Provides staking capabilities, reward distribution and reward management functionalities to the Phoenix DEX.```w

## Main methods:
#### 1. initialize

**params:**
* admin: `Address` of the administrator for the contract
* lp_token: `Address` of the liquidity pool used with this stake contract
* min_bond: `i128` value showing the minimum required bond
 * max_distributions: `u32` value showing the maximum number of distributions
 * min_reward: `i128` the minimum amount of rewards the user can withdraw.

**return type:**
void

**description:**
Used to set up the staking contract with the initial parameters.

<hr>

#### 2. bond

**params:**
* sender: `Address` of the user that sends tokens to the stake contract.
* tokens: `i128` value representing the number of tokens the user sends.

**return type:**
void

**description:**
Allows for users to stake/bond their lp tokens

<hr>

#### 3. unbond

**params:**
* sender: `Address` of the user that wants to unbond/unstake their tokens.
* stake_amount: `i128` value representing the numbers of stake to be unbond.
* stake_timestamp: `u64`value used to calculate the correct stake to be removed

**return type:**
void

**description:**
Allows the user remove their staked tokens from the stake contract, with any rewards they may have earned, based on the amount of staked tokens and stake's timestamp.

<hr>

#### 4. create_distribution_flow

**params:**
* sender: `Address` of the user that creates the flow
* manager: `Address` of the user that will be managing the flow
* asset: `Address` of the asset that will be used in the distribution flow

**return type:**
void

**description:**
Creates a distribution flow for sending rewards, that are managed by a  manager for a specific asset.

<hr>

#### 5. distribute_rewards

**params:**
* None

**return type:**
void

**description:**
Sends the rewards to all the users that have stakes, on the basis of the current reward distribution rule set and total staked amount.

<hr>

#### 6. withdraw_rewards

**params:**
* sender: `Address` of the user that wants to withdraw their rewards

**return type:**
void

**description:**
Allows for users to withdraw their rewards from the stake contract.

<hr>

#### 7. fund_distribution

**params:**
* sender: `Address` of the user that calls this method.
* start_time: `u64` value representing the time in which the funding has started.
* distribution_duration: `u64` value representing the duration for the distribution in seconds
* token_address: `Address` of the token that will be used for the reward distribution 
* token_amount: `i128` value representing how many tokens will be allocated for the distribution time


**return type:**
void

**description:**
Sends funds for a reward distribution.

<hr>

## Queries:
#### 1. query_config

**params:**
* None

**return type:**
`ConfigResponse` struct.

**description:**
Queries the contract `Config` 

<hr>

#### 2. query_admin

**params:**
* None

**return type:**
`Address` struct.

**description:**
Returns the address of the admin for the given stake contract. 

<hr>

#### 3. query_staked

**params:**
* address: `Address` of the stake contract we want to query

**return type:**
`StakedResponse` struct.

**description:**
Provides information about the stakes of a specific address. 

<hr>

#### 4. query_total_staked

**params:**
* None

**return type:**
`i128` 

**description:**
Returns the total amount of tokens currently staked in the contract. 

<hr>

#### 5. query_annualized_rewards

**params:**
* None

**return type:**
`AnnualizedRewardsResponse`  struct

**description:**
Provides an overview of the annualized rewards for each distributed asset. 

<hr>

#### 6. query_withdrawable_rewards

**params:**
* address: `Address` whose rewards we are searching

**return type:**
`WithdrawableRewardsResponse`  struct

**description:**
Queries the amount of rewards that a given address can withdraw. 

<hr>

#### 7. query_distributed_rewards

**params:**
* asset: `Address` of the token for which we query

**return type:**
`u128`

**description:**
Reports the total amount of rewards distributed for a specific asset.

<hr>

#### 8. query_undistributed_rewards

**params:**
* asset: `Address` of the token for which we query

**return type:**
`u128`

**description:**
Queries the total amount of remaining rewards for a given asset.
