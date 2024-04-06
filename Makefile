# 在rust里面使用makefile，确实很奇怪，但是makefile写起来很快，比写build.rs快。所以先这么写着吧

hd60M.img:
	cp empty60M.img build/hd60M.img

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
hd: mbr.bin loader.bin
	dd if=build/mbr.bin  of=build/hd60M.img bs=512 count=1 conv=notrunc && \
	dd if=build/loader.bin of=build/hd60M.img bs=512 count=4 seek=2 conv=notrunc

build: hd mbr.bin loader.bin

run: hd mbr.bin loader.bin
	qemu-system-x86_64 -drive format=raw,file=build/hd60M.img --full-screen

clean: 
	cargo clean && \
	cd build && rm -rf ./*

