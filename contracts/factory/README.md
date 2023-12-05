

# FACTORY

## Main functionality
```The main purpose of the factory contract is to provide the tooling required for managing, creating, querying of liquidity pools.```



## Main methods:
#### fn initialize(env: Env, admin: Address, multihop_wasm_hash: BytesN<32>);

**params:**
```
-- admin: Address of the contract administrator to be
-- multihop_wasm_hash: wasm hash of the multihop contract to be deployed initially
```
**return type:**
`void`

**description:**
`Used for the initialization of the factory contract - this sets the factory contract as initialized, deploys a multihop contract and sets up the initial factory configuration. The configuration includes admin and address of multihop contract`

<hr>

####  fn create_liquidity_pool(env: Env, lp_init_info: LiquidityPoolInitInfo) -> Address;
**params:**
```
-- lp_init_info: information for the new liquidity pool
```
**return type:**
`Address` of the newly created liquidity pool

**description:**
`Creates a new liquidity pool with 'LiquidityPoolInitInfo'. After deployment of the liquidity pool it updates the liquidity pool list.`

<hr>

#### fn query_pools(env: Env) -> Vec<Address>

**params:**
```
-- None
```
**return type:**
`Vec<Address>` of all the liquidity pools created by the factory

**description:**
`Queries for a list of all the liquidity pool addresses that have been created by the called factory contract.`

<hr>

#### fn query_pool_details(env: Env, pool_address: Address) -> LiquidityPoolInfo;

**params:**
```
-- pool_address: Address of the liquidity pool we search for
```
**return type:**
`LiquidityPoolInfo` Struct containing the information about a given liquidity pool.

**description:**
`Queries for specific liquidity pool information that has been created by the called factory contract.`

<hr>

#### fn query_all_pools_details(env: Env) -> Vec<LiquidityPoolInfo>;

**params:**
```
-- None
```
**return type:**
`Vec<LiquidityPoolInfo>` List of structs containing the information about all liquidity pools created by the factory.

**description:**
`Queries for all liquidity pools information that have been created by the called factory contract.`

<hr>

#### fn query_for_pool_by_token_pair(env: Env, token_a: Address, token_b: Address) -> Address;

**params:**
```
-- token_a: Address of the first token in the pool
-- token_b: Address of the second token in the pool
```
**return type:**
`Address` of the found liquidity pool that holds the given token pair.

**description:**
`Queries for a liquidity pool address by the tokens of that pool.`

<hr>

#### fn get_admin(env: Env) -> Address;

**params:**
```
-- None
```
**return type:**
`Address` of the admin for the called factory.

**description:**
`Queries for admin address of the called factory contract`
<hr>

####  fn get_config(env: Env) -> Config;

**params:**
```
-- None
```
**return type:**
`Config` of the called factory.

**description:**
`Queries for the config of the called factory contract`

