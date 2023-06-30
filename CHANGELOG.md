# CHANGELOG

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Curve: Add fully functional implementation of distribution curve that handles 3 types (constant, linear and piecewise linear) ([#62])

### Changed

- Decimal: use num_bigint crate to increase increase range of allowed values and prevent avoidable overflow; increase test coverage ([#55])
- Pair: modify swap signature to accept spread as BPS instead of plain number translated to percentage ([#56])
- Pair: implement update_config message ([#58])

[#55]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/55
[#56]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/56
[#58]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/58
[#62]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/pull/62

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

[unreleased]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/compare/v0.3.1...HEAD
[0.3.1]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/compare/v0.2.7...v0.3.1
[0.2.7]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/compare/v0.2.0...v0.2.7
[0.2.0]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/compare/04263245592bd2f4902766dfbc45d830e87570b1...v0.2.0
