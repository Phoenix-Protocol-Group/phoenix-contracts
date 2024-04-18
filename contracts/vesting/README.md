# Phoenix Protocol Vesting

This is the vesting contract for the Phoenix Protocol. It is implemented in Rust and compiled to WASM for execution on the Soroban platform.

## Functionality

- Initialize vesting with admin, vesting token, vesting balances, minter info, allowed vesters, and max vesting complexity.
- Transfer tokens from one address to another by checking if the transfer does not violate the vesting curve.
- Transfer vesting from one address to another with a specified curve, verifying the new curve combination.
- Burn tokens from a specified address.
- Mint tokens to a specified address.
- Update Minter with new address.
- Update Minter's capacity Curve.
