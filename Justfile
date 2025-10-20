# Defaults (override via env or on the CLI)
set shell := ["bash", "-cu"]

# These locations and file names vary per distribution.
# You can try to find them using `just find-ovmf`.
ovmf-dir := "/usr/share/OVMF"
ovmf-code-file := "OVMF_CODE_4M.fd"
ovmf-vars-file := "OVMF_VARS_4M.fd"

# Assembled paths for the OVMF UEFI code and variable templates.
ofmv-code-path := ovmf-dir / ovmf-code-file
ofmv-vars-path := ovmf-dir / ovmf-vars-file

# Where to package the local development files for QEMU runs.
build-local-dir := "qemu"
exp-local-dir := build-local-dir / "esp"
uefi-local-dir := exp-local-dir / "EFI/Boot"

# Where to store the local copy of the UEFI vars.
ofmv-local-vars-path := build-local-dir / "uefi-vars.fd"

# How to rename the example EFI binary for easier access.
uefi-local-file := "BootX64.efi"
uefi-local-path := uefi-local-dir / uefi-local-file

[private]
help:
    @just --list --unsorted

# Find the OFMF UEFI firmware for QEMU
find-ovmf:
    fd -HI OVMF_CODE.fd /usr/share 2>/dev/null || find /usr/share -name 'OVMF*.fd'

# Ensures the target directory exists.
[private]
_make-target-dir:
    @mkdir -p {{ uefi-local-dir }}

# Copy the OFMF UEFI vars to the local directory
reset-uefi-vars: _make-target-dir
    @rm {{ uefi-local-dir / "*.fd" }} || true
    @cp "{{ ofmv-vars-path }}" "{{ ofmv-local-vars-path }}"
    @echo "Updated {{ ofmv-local-vars-path }}"

# Package the build artifacts into the target dir
package FLAVOR="debug": reset-uefi-vars
    @rm {{ uefi-local-dir / "*.efi" }} || true
    @cp "target/x86_64-unknown-uefi/{{ FLAVOR }}/uefi-experiments.efi" "{{ uefi-local-path }}"
    @echo "Updated {{ uefi-local-path }}"

# Run the firmware in QEMU using OVMF (pass arguments like -nographic)
run-qemu *ARGS: package
    qemu-system-x86_64 \
      -machine q35 \
      -m 256 \
      -drive "if=pflash,format=raw,readonly=on,file={{ ofmv-code-path }}" \
      -drive "if=pflash,format=raw,file={{ ofmv-local-vars-path }}" \
      -drive "format=raw,file=fat:rw:{{ exp-local-dir }}" \
      -net none {{ ARGS }}

# Build for UEFI (see .cargo/config.toml for details)
build *ARGS:
    @cargo build {{ ARGS }}
