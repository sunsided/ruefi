# Changelog

All notable changes to this project will be documented in this file.
This project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

[Unreleased]: https://github.com/sunsided/uefi-experiments/compare/v0.1.0..HEAD

## Added

- Added `Justfile` commands for bundling the UEFI binary into an image file and
  running it off the image in QEMU.

## Internal

- Make error handling explicit and more consistent.

## [0.1.0] - 2025-10-20

[0.1.0]: https://github.com/sunsided/uefi-experiments/releases/tag/v0.1.0

### Added

- The application now waits for 5 seconds or a keystroke before exiting.
- We're now using an `esp` (EFI System Partition) for the UEFI binary to get
  our EFI application started automatically.
- Added initial console output example.
- Added `Justfile` for a minimal bootstrap, add example for running UEFI binary in QEMU.
