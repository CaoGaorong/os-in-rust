user.bin:
	cargo build --release && \
	cd .. && \
	x86_64-linux-gnu-objcopy -I elf64-x86-64 -O binary target/cat/release/cat build/cat.bin

compile: user.bin
	cd ../ && \
	dd if=build/cat.bin of=build/hd60M.img bs=512 count=100 seek=250 conv=notrunc