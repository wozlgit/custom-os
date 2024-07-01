SHELL=/bin/bash

run-qemu:
	qemu-system-x86_64 -machine q35 -cpu host -smp 4 -accel kvm -m 256M \
	-bios dependencies/edk2/Build/OvmfX64/RELEASE_GCC/FV/OVMF.fd \
	-device qemu-xhci \
	-blockdev driver=file,node-name=disk,filename=disk_image.bin \
	-device usb-storage,drive=disk \
	-net none

build:
	cargo build && \
	bash create_os_image_no_root.sh

run:
	make build && make run-qemu

build-release:
	cargo build --release && \
	bash create_os_image_no_root.sh

run-release:
	make build-release && make run-qemu

build-ovmf:
	cd dependencies/edk2 && \
	git submodule update --init && \
	make -C BaseTools && \
	source edksetup.sh && \
	cp ../edk2_target.txt Conf/target.txt && \
	build

clean:
	rm disk_image.bin && cargo clean
