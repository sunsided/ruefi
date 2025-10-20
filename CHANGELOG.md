# Changelog

All notable changes to this project will be documented in this file.
This project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

[Unreleased]: https://github.com/sunsided/uefi-experiments/compare/277456229a7d7ab07a82e250b514cb2683432a0c..HEAD
<!-- [0.1.0]: https://github.com/sunsided/uefi-experiments/releases/tag/v0.1.0 -->

### Added

- We're now using an `esp` (EFI System Partition) for the UEFI binary to get
  our EFI application started automatically. The application now waits for 3 seconds before exiting.
- Added initial console output example.
- Added `Justfile` for a minimal bootstrap, add example for running UEFI binary in QEMU.
