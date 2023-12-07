# FACTORY

## Main functionality
```The main purpose of the factory contract is to provide the tooling required for managing, creating, querying of liquidity pools.```



## Main methods:
#### 1. initialize

**params:**
* admin: Address of the contract administrator to be
* multihop_wasm_hash: wasm hash of the multihop contract to be deployed initially

**return type:**
 void

**description:**
Used for the initialization of the factory contract - this sets the factory contract as initialized, deploys a multihop contract and sets up the initial factory configuration. The configuration includes admin and address of multihop contract

<hr>

#### 2.  create_liquidity_pool
**params:**

* lp_init_info: information for the new liquidity pool

**return type:**
`Address` of the newly created liquidity pool

**description:**
Creates a new liquidity pool with 'LiquidityPoolInitInfo'. After deployment of the liquidity pool it updates the liquidity pool list.

<hr>

#### 3. query_pools

**params:**
 * None

**return type:**
`Vec<Address>` of all the liquidity pools created by the factory

**description:**
Queries for a list of all the liquidity pool addresses that have been created by the called factory contract.

<hr>

#### 4. query_pool_details

**params:**

* pool_address: Address of the liquidity pool we search for

**return type:**
`LiquidityPoolInfo` Struct containing the information about a given liquidity pool.

**description:**
Queries for specific liquidity pool information that has been created by the called factory contract.

<hr>

#### 5. query_all_pools_details

**params:**

* None

**return type:**
`Vec<LiquidityPoolInfo>` List of structs containing the information about all liquidity pools created by the factory.

**description:**
Queries for all liquidity pools information that have been created by the called factory contract.

<hr>

#### 6. query_for_pool_by_token_pair(env: Env, token_a: Address, token_b: Address) -> Address;

**params:**

* token_a: Address of the first token in the pool
* token_b: Address of the second token in the pool

**return type:**
`Address` of the found liquidity pool that holds the given token pair.

**description:**
Queries for a liquidity pool address by the tokens of that pool.

<hr>

#### 7. get_admin

**params:**

* None

**return type:**
`Address` of the admin for the called factory.

**description:**
Queries for admin address of the called factory contract
<hr>

####  8. get_config

**params:**

* None

**return type:**
`Config` of the called factory.

**description:**
Queries for the config of the called factory contract

