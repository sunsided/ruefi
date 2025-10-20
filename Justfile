ovmf-dir := "/usr/share/OVMF"
ovmf-code-file := "OVMF_CODE_4M.fd"
ovmf-vars-file := "OVMF_VARS_4M.fd"

ofmv-code-path := ovmf-dir / ovmf-code-file
ofmv-vars-path := ovmf-dir / ovmf-vars-file

ofmv-local-vars-file := "uefi-vars.fd"

[private]
help:
    @just --list --unsorted

# Find the OFMF UEFI firmware for QEMU
find-ovmf:
    fd -HI OVMF_CODE.fd /usr/share 2>/dev/null || find /usr/share -name 'OVMF*.fd'

# Copy the OFMF UEFI vars to the local directory
reset-uefi-vars:
    cp "{{ ofmv-vars-path }}" uefi-vars.fd

# Run the firmware in QEMU using OVMF, pass arguments like `-nographic`
run-qemu *ARGS: reset-uefi-vars
    qemu-system-x86_64 \
      -drive "if=pflash,format=raw,readonly=on,file={{ ofmv-code-path }}" \
      -drive "if=pflash,format=raw,file={{ ofmv-local-vars-file }}" \
      -drive format=raw,file=fat:rw:target/x86_64-unknown-uefi/debug \
      -net none {{ ARGS }}
