# CHANGELOG

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## Changed

- decimal: Rename to `soroban-decimal` and publish in crates.io ([#299])
- curve: try to optimize piecewise linear implementation ([#307])
- Update soroban-sdk-rs to version 20.5.0 ([#308])
- Pool and Pool Stable: adds verification if the current timestamp is after a desired timestamp ([#322])
- All: adds helper function that safely casts i128 to u128 ([#365])
- Pool and Pool stable: improves tracked balance when using fee on swap and provide liquidity ([#332])

## Added

- Factory: Allows creation of stable pools ([#301])
- Trader: Adds nominated trader contract to the DEX ([#288])
- Multihop: Allows multihop swaps for stable pool ([#303])
- Stake rewards: Create a new contract and prepare first testcases ([#306])
- Trader: Adds a check if the contract has been already initialized ([#329])
- All: Adds more test that verify the authorization during execution ([#363])
- Pool Stable: Adds a check to prevent creating pools with tokens with too big precision ([#360])

## Fixed

- Pool stable: Wrong calculation of swap constant reduces stable pool's efficiency ([#366])

[#288]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/288
[#299]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/299
[#301]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/301
[#303]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/303
[#306]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/306
[#307]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/307
[#308]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/308
[#322]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/322
[#332]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/332
[#329]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/329
[#360]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/360
[#363]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/363
[#365]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/365
[#366]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/366

## [1.0.0] - 2024-05-08

## Changed

- Token: Update the contract to latest version from soroban-examples ([#274])
- Update soroban-sdk-rs to 20.5.0 ([#275])
- CI: Update rust to 1.77.0 ([#275])
- Pool: Replace belief_price swap parameter with minimum amount of tokens expected to be received ([#280])
- Stake: Checks if curve complexity is too high during operations and also optimises the curve combine process ([#283])
- Pool: Temporarily disable the feature that allows to provide liquidity with a single token ([#289])
- Stake: Optimize the process of adding stake if done within 12 hour period between stakes ([#295])

## Added

- vesting: Add vesting contract ([#267])
- all: changes panic! to panic_with_error ([#269])
- Audit Report from Veridise ([#291])
- all: added entrypoint to update the contracts ([#293])
- Pool and Pool Stable: adds query for totally issued tokens ([#298])

[#267]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/267
[#269]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/269
[#275]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/275
[#274]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/274
[#280]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/280
[#283]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/283
[#288]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/288
[#289]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/289
[#291]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/291
[#293]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/293
[#295]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/295
[#298]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/298

## Bug fixes

## [0.9.0] - 2024-03-12

## Changed

- Stake: unbonding now automatically pays the rewards ([#170])
- Factory: add query displaying data for given user ([#176])
- Update soroban-sdk version from v20.0.3 to v20.1.0 ([#193])
- Fixes documentation and naming ([#200])
- Multihop: adds a new field in the Swap struct, that hold max_belief_price ([234])
- Fixes incorrect assignment of total_fee_bps in both pool and pool stable ([235])
- Pool: adds a missed part of return_amount argument ([#238])
- Pool: Replace panic! with panic_with_error! to provide more contextual information ([#206])
- Multihop: changes total_commission_amount type in SimulateSwap and SimulateReverseSwap ([#236])
- Multihop: removes unnecessary unwrap of a value ([#240])
- Factory, Multihop, Pool, Pool_stable, Phoenix: adds lp_token decimal's as a const instead a user input ([#241])
- Factory, Multihop, Pool, Pool_stable: adds a new parameter for creating liquidity pool ([#243])
- Update soroban-sdk version from v20.1.0 to v20.4.0 ([#262])

[#170]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/170
[#176]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/176
[#200]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/200
[#234]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/234
[#235]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/235
[#238]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/238
[#206]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/206
[#236]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/236
[#240]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/240
[#241]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/241
[#243]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/243
[#262]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/262

## Added

- Adds a new macro that validates the bps arguments value ([#199])
- Added test coverage for Decimal ([#244])

[#199]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/199
[#244]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/244

## Bug fixes

- Pool stable: Fixes an error in the compute_swap function, where commission isn't deducted ([233])
- Pool and Pool stable: extend check if the swapped token is within pool ([#237])
- Pool and Pool stable: adds missing validation of max_spread in do_swap ([#239])
- Pool: Adds a validation for when shares can be zero during withdrawal ([#245])
- Pool: adds new check that verifies the input params for get_deposit_amounts ([#246])
- Stake: Adds access control to create distribution flow ([#249])

[#233]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/233
[#237]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/237
[#239]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/239
[#245]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/245
[#246]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/246
[#249]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/249

## [0.8.0] - 2024-01-17

## Changed

- Factory: Replace tuple in a query `query_for_pool_by_pair_tuple` with a two separate parameters; rename that query to `query_for_pool_by_token_pair` ([#144])
- Total surrender refactor: replace all errors with panics ([#140])
- Pool/Pool stable: Replace `sell_a` parameter in simulate swap messages with `offer_asset` address ([#154])
- Multihop: Implement simulate swap/reverse swap queries ([#147])
- Factory: Initializes the Multihop contract upon initializing Factory ([#158])
- Multihop: Checks if the list of Swaps being sent is not empty / is valid ([#159])
- Multihop: Adds a new parameter to SimulateSwapResponse and SimulateReverseSwapResponse, that keeps information about spread ([#168])
- Temporarily disable the referral feature ([#191])
- Update soroban-sdk version to v20.0.3 ([#174])

## Bug fixes

- All: Adds a flag in all contract initialization functions to check if the contract has been already initialized ([#157])

[#140]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/140
[#144]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/144
[#147]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/147
[#157]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/157
[#158]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/158
[#159]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/159
[#168]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/168

## [0.7.0] - 2023-10-10

## Added

- Stake: Implement APR query ([#101])
- Multihop: Provide swap implementation and a testing framework ([#133])
- Multihop: Refactor swap algorithm and fix authorization issue on subsequent swaps ([#138])

## Changed

- Pair: Replace `sell_a` parameter in swap message with `offer_asset` address ([#141])
- Pool: Rename `pair` to `pool` to avoid further confusion in names ([#139])

[#101]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/101
[#133]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/133
[#138]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/138
[#141]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/141
[#139]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/139

## [0.6.0] - 2023-09-20

## Added

- Stake: Create new distribution structure for staking rewards flow ([#83])
- DEX deployment script ([#97])
- Stake: Implement full distribution mechanism for static rewards ([#88])
- Stable pair: Initialize contract ([#108])
- Stake: Added new variable TotalStaked in storage ([#94])
- Pair: Deployment of Stake contract during initialization ([#98])
- Factory: Contract that allows us to deploy Liquidity Pools ([#112])
- Decimal: Implement `from_atomics` and `to_string` ([#115])
- Curve: Implement `end` helper ([#115])
- Phoenix: Helper library for commonly used functions, structs, etc... ([#116])
- Factory: Adds functionality that provides more detailed information about a single pool or a vector of pools ([#128])
- Factory: Adds new query to search for a liquidity pool address by a tuple of token addresses. ([#131])

## Changed

- Curve: Modify implementation to not use named fields in Curve enum, since they are not allowed currently in soroban-sdk ([#86])
- Curve: Modify implementation of PiecewiseLinear type to avoid using tuple due to soroban-sdk limitations ([#100])
- Pair: Removes redundant slippage check ([#96])
- All: more granular error handling ([#95])
- All: changed the test for auths() and panic! ([#90])
- Factory: Remove pair initialization through client in order to not import other contract in binary ([#122])

## Fixed

- Stake: Reward distribution points are now updated when new user bonds ([#118])

[#83]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/83
[#86]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/86
[#88]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/88
[#90]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/90
[#94]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/94
[#95]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/95
[#96]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/96
[#97]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/97
[#98]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/98
[#100]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/100
[#108]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/108
[#112]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/112
[#115]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/115
[#116]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/116
[#118]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/118
[#122]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/122
[#128]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/128
[#131]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/131

## [0.5.0] - 2023-08-04

## Added

- Stake: Implement bonding and unbonding ([#78] [#79])

## Changed

- Update soroban-sdk from v0.8.4 to v0.9.2 ([#81])

[#78]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/78
[#79]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/79
[#81]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/81

## [0.4.0] - 2023-07-04

### Fixed

- Pair: Incorrect division of assets during providing liquidity with single token; solution was to implement binary search approximation algorithm that finds optimal division to keep the pool ratio intact (within 1%) ([#66])

### Added

- Curve: Add fully functional implementation of distribution curve that handles 3 types (constant, linear and piecewise linear) ([#62])
- CI: Upload CI results to codecov and implement coverage badge in the readme ([#64])
- Pair: input parameter validation (>= 0) ([#66])

### Changed

- Decimal: use num_bigint crate to increase increase range of allowed values and prevent avoidable overflow; increase test coverage ([#55])
- Pair: modify swap signature to accept spread as BPS instead of plain number translated to percentage ([#56])
- Pair: implement update_config message ([#58])
- Decimal: Replace [wee-alloc](https://github.com/rustwasm/wee_alloc) in favor of soroban allo features ([#63])

[#55]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/55
[#56]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/56
[#58]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/58
[#62]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/62
[#63]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/63
[#64]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/64
[#66]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/66

## [0.3.1] - 2023-06-27

### Added

- Pair: implement checksum in release artifacts ([#37])
- Pair: implement withdraw endpoint in pair contract ([#38])
- Pair: implement protocol fees that are subtracted during the swap ([#40])
- Pair: implement slippage tolerance into pair contract ([#41])
- Pair: implement swap/reverse swap simulation ([#45])
- Pair: implement upgrade entrypoint ([#46])
- Pair: implement single asset liqudity providing ([#50])

### Changed

- Decimal: replace u128 with i128 in Decimal crate implementation... because someone sometime might want to use negative numbers in their contracts ([#38])
- improved architecture docs ([#44])

[#37]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/37
[#38]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/38
[#40]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/40
[#41]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/41
[#44]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/44
[#45]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/45
[#46]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/46
[#50]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/50

## [0.2.7] - 2023-06-22

### Added

- CI: automated release job which will create artifacts after new tag publication ([#29])
- standarized Makefiles across whole repository ([#29])

### Changed

- better error support for pair contract; introduced logs and events ([#28])

[#28]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/28
[#29]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/29

## [0.2.0] - 2023-06-20

### Added

- initial architecture draft ([#2])
- rust workspace with working CI ([#5])
- implement crate to easier manipulate Decimal types ([#21])
- XYK pair swap implementation ([#19])
- pool state queries ([#24])

[#2]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/2
[#5]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/5
[#21]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/21
[#19]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/19
[#24]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/24

[unreleased]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/compare/v0.9.0...v1.0.0
[0.9.0]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/compare/v0.8.0...v0.9.0
[0.8.0]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/compare/v0.3.1...v0.4.0
[0.3.1]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/compare/v0.2.7...v0.3.1
[0.2.7]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/compare/v0.2.0...v0.2.7
[0.2.0]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/compare/04263245592bd2f4902766dfbc45d830e87570b1...v0.2.0
