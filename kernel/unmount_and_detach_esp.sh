#!/bin/bash
sudo umount efi_partition_dir
# ONLY FOR TESTING PURPOSES, this will detach all loop devices on the system, not just the one containing the ESP!
sudo losetup -D
