# Phoenix Protocol Vesting

This is the vesting contract for the Phoenix Protocol. It is implemented in Rust and compiled to WASM for execution on the Soroban platform.

## Key Features

- Initialize vesting with admin, vesting token, vesting balances, minter info, allowed vesters, and max vesting complexity.
- Transfer tokens from one address to another.
- Transfer vesting from one address to another with a specified curve.
- Burn tokens from a specified address.
- Mint tokens to a specified address.

## Key Functions

- `initialize(env: Env, admin: Address, vesting_token: VestingTokenInfo, vesting_balances: Vec<VestingBalance>, minter_info: Option<MinterInfo>, allowed_vesters: Option<Vec<Address>>, max_vesting_complexity: u32) -> Result<(), ContractError>`
- `transfer_token(env: Env, from: Address, to: Address, amount: i128) -> Result<(), ContractError>`
- `transfer_vesting(env: Env, from: Address, to: Address, amount: i128, curve: Curve) -> Result<(), ContractError>`
- `burn(env: Env, sender: Address, amount: i128) -> Result<(), ContractError>`
- `mint(env: Env, sender: Address, to: Address, amount: i128)`

## Dependencies

- `curve`: Used for managing vesting curves.
- `soroban_sdk`: Used for contract implementation and utilities.

## Testing

Run tests with either `make test` or `cargo test`.

## License

This project is licensed under the terms of the included LICENSE file.