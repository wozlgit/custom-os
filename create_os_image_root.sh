#!/bin/bash

if ! [ -e disk_image.dd ]; then
    dd if=/dev/zero of=disk_image.dd bs=$[1024*1024] count=64
    fdisk disk_image.dd
    sudo losetup -o $[2048*512] --sizelimit 50M -f disk_image.dd
    sudo mkfs.fat -F 32 -n "EFI System" /dev/loop0
else
    sudo losetup -o $[2048*512] --sizelimit 50M -f disk_image.dd
fi
mkdir -p efi_partition_dir 
# Assuming here that /dev/loop0 is the name of the loop device created earlier, which very much might be wrong
sudo mount /dev/loop0 efi_partition_dir

sudo mkdir -p efi_partition_dir/EFI/BOOT
sudo cp dependencies/limine/BOOTX64.EFI efi_partition_dir/EFI/BOOT/BOOTx64.EFI
sudo cp target/bare_metal_x86_64_target/debug/custom_os efi_partition_dir/kernel
sudo cp limine.cfg efi_partition_dir/

sudo umount efi_partition_dir
sudo losetup -d /dev/loop0
