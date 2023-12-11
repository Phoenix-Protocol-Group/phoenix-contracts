# Dex Factory

## Main functionality

The main purpose of the factory contract is to provide the tooling required for managing, creating, querying of liquidity pools.

## Messages

`initialize`

Params:
- `admin`: `Address` of the contract administrator to be
- `multihop_wasm_hash`: `BytesN<32>` hash of the multihop contract to be deployed initially

<hr>

`create_liquidity_pool`

Params:
- `lp_init_info`: `LiquidityPoolInitInfo` struct representing information for the new liquidity pool

Return type:
`Address` of the newly created liquidity pool

Description:

Creates a new liquidity pool with 'LiquidityPoolInitInfo'. After deployment of the liquidity pool it updates the liquidity pool list.

<hr>

`query_pools`

Return type:
`Vec<Address>` of all the liquidity pools created by the factory

Description:
Queries for a list of all the liquidity pool addresses that have been created by the called factory contract.

<hr>

`query_pool_details`

Params:
- `pool_address`: `Address` of the liquidity pool we search for

Return type:
Struct `LiquidityPoolInfo` containing the information about a given liquidity pool.

Description:
Queries for specific liquidity pool information that has been created by the called factory contract.

<hr>

`query_all_pools_details`

Return type:
`Vec<LiquidityPoolInfo>` list of structs containing the information about all liquidity pools created by the factory.

Description:
Queries for all liquidity pools information that have been created by the called factory contract.

<hr>

`query_for_pool_by_token_pair(env: Env, token_a: Address, token_b: Address)`;

Params:
- token_a: `Address` of the first token in the pool
- token_b: `Address` of the second token in the pool

Return type:
`Address` of the found liquidity pool that holds the given token pair.

Description:
Queries for a liquidity pool address by the tokens of that pool.

<hr>

`get_admin`

Return type:
`Address` of the admin for the called factory.

<hr>

`get_config`

Return type:
Struct `Config` of the called factory.
