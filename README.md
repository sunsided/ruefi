# üfi

Experiments with UEFI.

## Setup

```shell
rustup target add x86_64-unknown-uefi
sudo apt install qemu-system ovmf
```

## Justfile commands

- `just build`: Build the application in `debug` flavor
- `just run-qemu`: Run the application in QEMU`

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
