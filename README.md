[![codecov](https://codecov.io/gh/Phoenix-Protocol-Group/phoenix-contracts/branch/main/graph/badge.svg?token=BJMG2IINQB)](https://codecov.io/gh/Phoenix-Protocol-Group/phoenix-contracts)

# Phoenix DEX Smart Contracts
This repository contains the Rust source code for the smart contracts of the Phoenix DEX.

## Overview
Phoenix will be a set of DeFi protocols hosted on Soroban platform. Directory `docs` contains brief description of architecture, including flow diagrams.

## Prerequisites
The following tools are required for compiling the smart contracts:

- Rust ([link](https://www.rust-lang.org/tools/install))
- make

```bash
# Install rust using rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# Then install wasm32 target
rustup target add wasm32-unknown-unknown
```

## Compilation
The smart contracts can be compiled into WebAssembly (WASM) using make. The Makefile included in the repository is configured to handle the necessary steps for building the smart contracts.

Navigate to the root directory of the project in your terminal and run the following command:

```bash
make build
```

This will generate WASM files for each of the smart contracts in the `target/wasm32-unknown-unknown/release/` directory.

## Testing
You can run tests with the following command:

```bash
make test
```

## License
The smart contracts and associated code in this repository are licensed under the GPL-3.0 License. By contributing to this project, you agree that your contributions will also be licensed under the GPL-3.0 license.

For the full license text, please see the LICENSE file in the root directory of this repository.

## Contact
If you have any questions or issues, please create a new issue on the GitHub repository.
