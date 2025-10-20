# 🦀🖥️ ~~üfi~~ UEFI

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

Experiments with UEFI.

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

## Example Output

When run with `just run-qemu` (or `just run-qemu -nographic` for headless):

```
Hello, world from bare-metal UEFI (Rust)!
Press any key or wait 5s…

Timeout reached. Goodbye!
```

To quit from headless mode, press `Ctrl-A x`. To quit from interactive mode, press `Ctrl-Shift-Q` (
or `Ctrl-Shift-A` to detach from input capture).
