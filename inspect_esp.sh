#!/bin/bash
mkdir -p efi_partition_dir
sudo losetup -f -o $[2048*512] --sizelimit 64M disk_image.bin
sudo mount /dev/loop0 efi_partition_dir/
