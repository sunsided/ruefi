# üfi

Experiments with UEFI.

## Setup

```shell
rustup target add x86_64-unknown-uefi
sudo apt install qemu-system ovmf
```

## Example Output

When run with `just run-qemu` (or `just run-qemu -nographic` for headless):

```
UEFI Interactive Shell v2.2
EDK II
UEFI v2.70 (Ubuntu distribution of EDK II, 0x00010000)
Mapping table
      FS0: Alias(s):HD0a1:;BLK1:
          PciRoot(0x0)/Pci(0x1,0x1)/Ata(0x0)/HD(1,MBR,0xBE1AFDFA,0x3F,0xFBFC1)
     BLK0: Alias(s):
          PciRoot(0x0)/Pci(0x1,0x1)/Ata(0x0)
     BLK2: Alias(s):
          PciRoot(0x0)/Pci(0x1,0x1)/Ata(0x0)
Press ESC in 1 seconds to skip startup.nsh or any other key to continue.
Shell> fs0:
FS0:\> uefi-experiments.efi
Hello, world from Rust UEFI!
Hello via with_stdout()
FS0:\> QEMU: Terminated
```

To quit from headless mode, press `Ctrl-A x`. To quit from interactive mode, press `Ctrl-Shift-Q` (
or `Ctrl-Shift-A` to detach from input capture).
