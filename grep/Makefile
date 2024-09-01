grep.bin:
	cargo build --release && \
	cd .. && \
	x86_64-linux-gnu-objcopy -I elf64-x86-64 -O binary target/grep/release/grep build/grep.bin

compile: grep.bin
	cd ../ && \
	dd if=build/grep.bin of=build/hd60M.img bs=512 count=50 seek=350 conv=notrunc