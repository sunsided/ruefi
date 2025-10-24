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

# Where to store the local copy of the UEFI vars.
ofmv-local-vars-path := build-local-dir / "uefi-vars.fd"

# Where to package the local development files for QEMU runs.
build-local-dir := "qemu"
esp-local-dir := build-local-dir / "esp"
uefi-local-dir := esp-local-dir / "EFI/Boot"

# How to rename the example EFI binary for easier access.
uefi-local-file := "BootX64.efi"
uefi-local-path := uefi-local-dir / uefi-local-file

# Where to store the image
uefi-image-file := "uefi.img"
uefi-image-path := build-local-dir / "uefi.img"

[private]
help:
    @just --list --unsorted

# Format the code
fmt:
    @cargo fmt --all

# Find the OFMF UEFI firmware for QEMU
find-ovmf:
    fd -HI OVMF_CODE.fd /usr/share 2>/dev/null || find /usr/share -name 'OVMF*.fd'

# Ensures the target directory exists.
[private]
_make-target-dir:
    @mkdir -p {{ uefi-local-dir }}

# Clean everything
clean:
    @cargo clean
    rm -rf "{{ build-local-dir }}"

# Copy the OFMF UEFI vars to the local directory
reset-uefi-vars: _make-target-dir
    @rm {{ build-local-dir / "*.fd" }} || true
    @cp "{{ ofmv-vars-path }}" "{{ ofmv-local-vars-path }}"
    @echo "Updated {{ ofmv-local-vars-path }}"

# Package the build artifacts into the target dir
package FLAVOR="release": reset-uefi-vars
    @rm {{ uefi-local-dir / "*.efi" }} || true
    @cp "target/x86_64-unknown-uefi/{{ FLAVOR }}/ruefi.efi" "{{ uefi-local-path }}"
    @echo "Updated {{ uefi-local-path }}"

# Build for UEFI (see .cargo/config.toml for details)
build *ARGS: fmt
    @cargo build --release {{ ARGS }}

# Build a disk image with ESP
build-img: build package
    rm "{{ uefi-image-path }}" || true

    # make a 64 MiB raw image
    truncate -s 64M "{{ uefi-image-path }}"

    # partition + ESP GUID + FAT32 format
    guestfish -x -a "{{ uefi-image-path }}" -- \
      run : \
      part-init /dev/sda gpt : \
      part-add /dev/sda p 2048 -34 : \
      part-set-gpt-type /dev/sda 1 c12a7328-f81f-11d2-ba4b-00a0c93ec93b : \
      mkfs vfat /dev/sda1 label:EFI : \
      exit

    # Mount ESP and copy BootX64.efi
    mkdir -p mnt
    guestmount -a "{{ uefi-image-path }}" -m /dev/sda1 mnt
    mkdir -p mnt/EFI/Boot
    cp "{{ uefi-local-path }}" mnt/EFI/Boot/BootX64.efi
    guestunmount mnt
    rmdir mnt

# Run the firmware in QEMU using OVMF (pass arguments like -nographic)
run-qemu *ARGS: package
    qemu-system-x86_64 \
      -machine q35 \
      -m 256 \
      -drive "if=pflash,format=raw,readonly=on,file={{ ofmv-code-path }}" \
      -drive "if=pflash,format=raw,file={{ ofmv-local-vars-path }}" \
      -drive "format=raw,file=fat:rw:{{ esp-local-dir }}" \
      -net none {{ ARGS }}

# Run the firmware in QEMU from an image file (created with build-img-x64)
run-qemu-img *ARGS:
    qemu-system-x86_64 \
      -machine q35 \
      -m 256 \
      -drive "if=pflash,format=raw,readonly=on,file={{ ofmv-code-path }}" \
      -drive "if=pflash,format=raw,file={{ ofmv-local-vars-path }}" \
      -drive "file={{ uefi-image-path }},if=virtio,format=raw" \
      -net none {{ ARGS }}

# Create distributable zip with checksum
dist: build-img
    mkdir dist
    rm -f dist/ruefi.zip dist/ruefi.zip.sha256 || true
    zip -j dist/ruefi.zip "{{ uefi-image-path }}"
    cd dist && sha256sum ruefi.zip > ruefi.zip.sha256
    @echo "Created ruefi.zip and ruefi.zip.sha256"
