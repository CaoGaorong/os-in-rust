user.bin:
	cargo build --release && \
	cd .. && \
	x86_64-linux-gnu-objcopy -I elf64-x86-64 -O binary target/user/release/user build/user.bin

compile: user.bin
	cd ../ && \
	dd if=build/user.bin of=build/hd60M.img bs=512 count=10 seek=300 conv=notrunc