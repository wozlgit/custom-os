# Building
## Build environment
### Dependencies for building EDK2
Voidlinux command: 'sudo xbps-install -Sy libuuid-devel nasm acpica-utils python'

### Dependencies for creating the kernel image
Voidlinux command: 'sudo xbps-install -Sy gptfdisk mtools'

## Build process
Before creating the bootable image file for the first time, the OVMF UEFI-firmware must be built. This can be done with the command
'make build-ovmf', executed from the kernel root directory. This *should* be a painless process requiring no further input.
    Then the bootable image file can be created by executing the 'create_os_image_no_root.sh' script, after having built the kernel code ('cargo build').
The 'make build' and derivative commands already do this. After the bootable image file has been created, it can be executed inside QEMU with the 'make run-qemu' command.
