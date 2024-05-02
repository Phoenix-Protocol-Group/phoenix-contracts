This is the designated trader contract, which is responsible for accumulating fees from each trading pair and converting them into the $PHO token. The contract is initialized with an admin, who is responsible for adding trade routes and configuring trading pairs.

**Messages**

`initialize(env: Env, admin: Address, contract_name: String, pair_addresses: (Address, Address), output_token: Address, max_spread: Option<u64>)`

Initializes the contract with the given admin and configuration.

`trade_token(env: Env, token_address: Address, liquidity_pool: Address, amount: Option<u64>)`

Performs a trade using the provided liquidity pool's contract. Only available tokens are traded.

`transfer(env: Env, recipient: Address, amount: u64, token_address: Option<Address>)`

Transfers tokens between addresses. Admins can withdraw PHO tokens to a given recipient.

**Queries**

`query_balances(env: Env)`

Returns the balances of all supported tokens.

`query_trading_pairs(env: Env)`

Returns the list of trading pairs configured for this contract.

`query_admin_info(env: Env)`

Returns information about the admin of this contract, including their address and permissions.

`query token_info(env: Env)`

Returns information about the output token, including its address and balance.
