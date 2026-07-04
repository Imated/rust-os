KERNEL  := target/x86_64-os/debug/rust-os
ISO     := kernel.iso
ISO_DIR := iso_root
LIMINE  := limine

.PHONY: all iso run run-uefi clean build

all: iso

build:
	cargo build

iso: build
	rm -rf $(ISO_DIR)
	mkdir -p $(ISO_DIR)/boot/limine $(ISO_DIR)/EFI/BOOT
	cp $(KERNEL)                     $(ISO_DIR)/boot/kernel
	cp misc/limine.conf              $(ISO_DIR)/boot/limine/limine.conf
	cp $(LIMINE)/limine-bios.sys     $(ISO_DIR)/boot/limine/
	cp $(LIMINE)/limine-bios-cd.bin  $(ISO_DIR)/boot/limine/
	cp $(LIMINE)/limine-uefi-cd.bin  $(ISO_DIR)/boot/limine/
	cp $(LIMINE)/BOOTX64.EFI        $(ISO_DIR)/EFI/BOOT/
	xorriso -as mkisofs \
		-b boot/limine/limine-bios-cd.bin \
		-no-emul-boot -boot-load-size 4 -boot-info-table \
		--efi-boot boot/limine/limine-uefi-cd.bin \
		-efi-boot-part --efi-boot-image \
		--protective-msdos-label \
		$(ISO_DIR) -o $(ISO)
	$(LIMINE)/limine bios-install $(ISO)

run: iso
	qemu-system-x86_64 \
		-bios /usr/share/edk2/x64/OVMF.4m.fd \
		-cdrom $(ISO) \
		-m 256M \
		-serial stdio -no-reboot -no-shutdown -d int -D logfile.txt

clean:
	cargo clean
	rm -rf $(ISO_DIR) $(ISO)