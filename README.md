# 🦀🖥️ Rüfi: UEFI in Rust

<!-- Shields -->
<p align="left">
	<a href="https://www.rust-lang.org/">
		<img src="https://img.shields.io/badge/Rust-2024-brightgreen.svg?logo=rust" alt="Rust">
	</a>
	<a href="https://uefi.org/">
		<img src="https://img.shields.io/badge/UEFI-Firmware-blue.svg?logo=uefi" alt="UEFI">
	</a>
	<a href="https://www.qemu.org/">
		<img src="https://img.shields.io/badge/QEMU-Emulator-orange.svg?logo=qemu" alt="QEMU">
	</a>
	<a href="https://github.com/tianocore/tianocore.github.io/wiki/OVMF">
		<img src="https://img.shields.io/badge/OVMF-UEFI%20Firmware-yellow.svg?logo=ovmf" alt="OVMF">
	</a>
  <a href="https://github.com/rust-secure-code/safety-dance/">
    <img src="https://img.shields.io/badge/unsafe-allowed-orange.svg" alt="unsafe allowed">
  </a>
</p>

This repository contains a toy project for implementing an [UEFI](https://en.wikipedia.org/wiki/UEFI) application
in Rust mainly by using the [uefi](https://crates.io/crates/uefi) crate. It
experiments with using the UEFI boot services, specifically the
Graphics Output Protocol (GOP) and Input devices, and provides
utility scripts for bundling the binary into an EFI System Partition (ESP)
or a flashable image. The example can be run directly in QEMU from
either source.

## Running it

Build a local ESP directory and boot from it using QEMU:

```shell
just build && just run-qemu
```

Alternatively, build an image and boot from it:

```shell
just build-img && just run-qemu-img
```

## Asteroids in UEFI

When run with `just run-qemu`:

![Screenshot](docs/screenshot.png)

* Arrow keys for movement
* Space key for firing
* Brackets (`[`, `]`) for changing projectile speed
* ESC to exit to UEFI

To quit from QEMU interactive mode, press `Ctrl-Shift-Q`
(or `Ctrl-Shift-A` to detach from input capture).

## Setup

```shell
rustup target add x86_64-unknown-uefi
sudo apt install qemu-system ovmf libguestfs-tools
```

## Justfile commands

Running off a directory mount:

- `just build`: Build the application in `debug` flavor
- `just run-qemu`: Run the application in QEMU`

With image files:

- `just build-img`: Build the application in `release` flavor and create a UEFI image
- `just run-qemu-img`: Run the application in QEMU from the UEFI image

Additional commands used internally but provided for convenience:

- `just package`: Package the application into an `esp` partition and prepare UEFI variables; called internally.
