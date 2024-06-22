# 在rust里面使用makefile，确实很奇怪，但是makefile写起来很快，比写build.rs快。所以先这么写着吧

hd60M.img:
	cp emtpy60M.img build/hd60M.img

hd80M.img:
	cp empty80M.img build/hd80M.img

mbr.bin:
	cd mbr && \
	cargo build --release && \
	cd .. && \
	x86_64-linux-gnu-objcopy -I elf64-x86-64 -O binary target/mbr/release/mbr build/mbr.bin

loader.bin:
	cd loader && \
	cargo build --release && \
	cd .. && \
	x86_64-linux-gnu-objcopy -I elf64-x86-64 -O binary target/loader/release/loader build/loader.bin

loader2.bin:
	cd loader2 && \
	cargo build --release && \
	cd .. && \
	x86_64-linux-gnu-objcopy -I elf64-x86-64 -O binary target/loader2/release/loader2 build/loader2.bin

kernel.bin:
	cd kernel && \
	cargo build --release && \
	cd .. && \
	x86_64-linux-gnu-objcopy -I elf64-x86-64 -O binary target/kernel/release/kernel build/kernel.bin


hd: hd60M.img mbr.bin loader.bin loader2.bin kernel.bin hd80M.img
	dd if=build/mbr.bin  of=build/hd60M.img bs=512 count=1 conv=notrunc && \
	dd if=build/loader.bin of=build/hd60M.img bs=512 count=1 seek=2 conv=notrunc && \
	dd if=build/loader2.bin of=build/hd60M.img bs=512 count=4 seek=3 conv=notrunc && \
	dd if=build/kernel.bin of=build/hd60M.img bs=512 count=200 seek=7 conv=notrunc

build: hd mbr.bin loader.bin loader2.bin kernel.bin

run: build
	qemu-system-x86_64 \
	-drive format=raw,file=build/hd60M.img \


debug: build
	qemu-system-i386 \
	-hda build/hd60M.img \
	-hdb build/hd80M.img \
	-S -s


clean: 
	cargo clean && \
	cd build && rm -rf ./*

