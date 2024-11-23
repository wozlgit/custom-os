#!/bin/bash
if ! [ -e disk_image.bin ]; then
    # Create 128 MiB disk image file filled with zeroes
    dd if=/dev/zero of=disk_image.bin bs=$[1024*1024] count=128
    # Create a GPT partition table in the disk image file, and a partition with EFI System partition OsType of size 64 MiB
    # The GPT header and partition tables together use up only the first 34 sectors of the disk, but all partitions must
    # be aligned on 1 MiB (which equals 2048 sectors on disks with 512-byte logical block size) boundaries
    sgdisk disk_image.bin -n 1:1M:65M -t 1:C12A7328-F81F-11D2-BA4B-00A0C93EC93B
    # Format the ESP as FAT32
    mformat -i disk_image.bin@@$[2048*512] -v "EFI System" -F
    # Create a /EFI/BOOT dir in the ESP
    mmd -i disk_image.bin@@$[2048*512] ::EFI ::EFI/BOOT
fi

# And finally copy all the needed files into the ESP
# The -D O and -D o options specify that the copied files should overwrite the old ones in the case of secondary
# and primary name conflicts, respectively
mcopy -i disk_image.bin@@$[2048*512] -D O -D o target/bare_metal_x86_64_target/debug/custom_os ::kernel # Kernel executable
# The UEFI specification mandates that the name of the bootloader is BOOTx64.EFI, although I'm not sure
# whether file names are even case sensitive in FAT32
mcopy -i disk_image.bin@@$[2048*512] -D O -D o dependencies/limine/BOOTX64.EFI ::EFI/BOOT/BOOTx64.EFI # Bootloader executable
mcopy -i disk_image.bin@@$[2048*512] -D O -D o limine.cfg :: # Limine configuration file
