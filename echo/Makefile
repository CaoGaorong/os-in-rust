user.bin:
	cargo build --release && \
	cd .. && \
	x86_64-linux-gnu-objcopy -I elf64-x86-64 -O binary target/echo/release/echo build/echo.bin

compile: user.bin
	cd ../ && \
	dd if=build/echo.bin of=build/hd60M.img bs=512 count=100 seek=400 conv=notrunc