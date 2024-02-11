SHELL=/bin/bash

build:
	cargo build && \
	bash create_os_image_no_root.sh

run:
	make build && \
	qemu-system-x86_64 -bios dependencies/edk2/Build/OvmfX64/RELEASE_GCC/FV/OVMF.fd \
	-net none -drive format=raw,file=disk_image.bin,media=disk

build-ovmf:
	cd dependencies/edk2 && \
	git submodule update --init && \
	make -C BaseTools && \
	source edksetup.sh && \
	cp ../edk2_target.txt Conf/target.txt && \
	build

clean:
	rm disk_image.bin && cargo clean
