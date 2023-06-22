# CHANGELOG

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- automated release job which will create artifacts after new tag publication ([#29])
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

[unreleased]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/Phoenix-Protocol-Group/phoenix-contracts/compare/04263245592bd2f4902766dfbc45d830e87570b1...v0.2.0
