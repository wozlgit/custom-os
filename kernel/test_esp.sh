#!/bin/bash
for i in {0..100}
do
    echo $i && \
    cargo build &> /dev/null && bash create_os_image_no_root.sh && \
    bash inspect_esp.sh && \
    sudo filefrag -e efi_partition_dir/kernel && \
    bash unmount_and_detach_esp.sh && \
    # This sed command adds the println!() statement
    # sed -i '87i\    println!("A");' src/main.rs
    # And this one removes it
    sed -i '87d' src/main.rs
done
