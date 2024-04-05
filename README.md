在[生成二进制大小问题](https://www.yuque.com/caogaorong/gxhc23/sfr3vi0xue6cf7kd?view=doc_embed)说到，我们在使用`cargo build`的时候可以使用`--release`来让生成的二进制更小，但是我发现使用了`--release`后，程序居然有很大的变动，导致无法正常运行。
<a name="TknJD"></a>
# 一、项目结构
我们目前还是一个简易的操作系统bootloader，有如下几个核心文件：

BIOS的mbr文件`boot.s`：
```sass
.section .boot, "awx"
.global _start
.code16

# This stage initializes the stack, enables the A20 line

_start:
    # zero segment registers
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov fs, ax
    mov gs, ax

    # clear the direction flag (e.g. go forward in memory when using
    # instructions like lodsb)
    cld

    # initialize stack
    mov sp, 0x7c00

enable_a20:
    # enable A20-Line via IO-Port 92, might not work on all motherboards
    in al, 0x92
    test al, 2
    jnz enable_a20_after
    or al, 2
    and al, 0xFE
    out 0x92, al
enable_a20_after:

check_int13h_extensions:
    push 'y'    # error code
    mov ah, 0x41
    mov bx, 0x55aa
    # dl contains drive number
    int 0x13
    # jc fail
    pop ax      # pop error code again

clear_screen:
   mov     ax, 0x0600
   mov     bx, 0x0700
   mov     cx, 0 
   mov     dx, 0x184f
   int     0x10


rust:
    call main

spin:
    hlt
    jmp spin

```
rust写的核心文件`main.rs`：
```rust
#![no_std]
#![no_main]
use core::{
    arch::{asm, global_asm},
    panic::PanicInfo,
};
global_asm!(include_str!("boot.s"));

#[no_mangle]
pub extern "C" fn main() -> ! {
    let vga_buffer = 0xb8000 as *mut u8;

    unsafe {
        *vga_buffer.offset(0) = b'H';
        *vga_buffer.offset(1) = 0xb;
        *vga_buffer.offset(2) = b'e';
        *vga_buffer.offset(3) = 0xb;
        *vga_buffer.offset(4) = b'l';
        *vga_buffer.offset(5) = 0xb;
        *vga_buffer.offset(6) = b'l';
        *vga_buffer.offset(7) = 0xb;
        *vga_buffer.offset(8) = b'0';
        *vga_buffer.offset(9) = 0xb;
    }

    loop {}
}

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

```
链接器文件`linker.ld`：
```rust
ENTRY(_start)

SECTIONS {
    . = 0x7c00;
    .boot :
    {
        *(.boot .boot.*)
    }
    .text :
    {
        *(.text .text.*)
    }
    .rodata :
    {
        *(.rodata .rodata.*)
    }
    .data :
    {
        *(.rodata .rodata.*)
        *(.data .data.*)
        *(.got .got.*)
    }

    . = 0x7c00 + 510;

    .magic_number :
    {
        SHORT(0xaa55)       /* magic number for bootable disk */
    }
}

```
rust项目的编译脚本`build.rs`：
```rust
fn main() {
    println!("cargo:rustc-link-arg=-Tlinker.ld");
}
```
<a name="XWZB6"></a>
# 二、cargo build 正常运行
我们先打包：
```rust
cargo build
```
可以得到`target/os-in-rust/debug/os-in-rust`文件：
```rust
> ll target/os-in-rust/debug/os-in-rust

-rwxr-xr-x  1 jackson  staff   7.8K Apr  5 14:45 target/os-in-rust/debug/os-in-rust
```
然后我们使用objcopy提取出纯二进制，得到`os-in-rust.bin`文件。
```rust
objcopy -I elf64-x86-64 -O binary target/os-in-rust/debug/os-in-rust os-in-rust.bin
```
我们使用qemu运行：
```rust
qemu-system-x86_64 -drive format=raw,file=os-in-rust.bin --full-screen
```
成功展示：

![image.png](images/1.png)


# 三、cargo build --release问题
我们现在打release包：
```rust
cargo build --release
```
可以得到`target/os-in-rust/release/os-in-rust`包：
```rust
> ll target/os-in-rust/release/os-in-rust
-rwxr-xr-x  1 jackson  staff   4.2K Apr  5 14:47 target/os-in-rust/release/os-in-rust
```
可以看到我们的release包已经比debug包小了一半。

然后我们使用objcopy提取二进制文件，得到`os-in-rust-release.bin`文件
```rust
objcopy -I elf64-x86-64 -O binary target/os-in-rust/release/os-in-rust os-in-rust-release.bin
```
然后我们使用qemu运行：
```rust
qemu-system-x86_64 -drive format=raw,file=os-in-rust-release.bin --full-screen
```
却发现输出是纯黑的，如下图所示：

![image.png](images/2.png)

<a name="oljGM"></a>
# 四、debug包和release包比较
我们来比较生成debug包和release包。
<a name="wojDw"></a>
## 大小
看看大小：
```shell
> ll target/os-in-rust/debug/os-in-rust
-rwxr-xr-x  1 jackson  staff   7.8K Apr  5 14:45 target/os-in-rust/debug/os-in-rust

> ll target/os-in-rust/release/os-in-rust
-rwxr-xr-x  1 jackson  staff   4.2K Apr  5 14:47 target/os-in-rust/release/os-in-rust
```
可以看到release包比debug包小很多。
<a name="UrHiY"></a>
## dump文件
我们再来看下debug包和release包的ELF文件内容。

我们使用`objdump`来查看两个包生成的反汇编文件：
```shell
> objdump -M intel -d target/os-in-rust/release/os-in-rust > release.dump
> objdump -M intel -d target/os-in-rust/debug/os-in-rust > debug.dump
```
我们通过比较`debug.dump`和`release.dump`文件，如下图：

![image.png](images/3.png)

可以看出，release包的main函数相对于debug包的main函数，少了很多内容。

<a name="OF5HG"></a>
## 二进制内容
如果我们不比较dump文件，直接比较二进制内容，也可以很直观地看出来两者的差距：

![image.png](images/4.png)

<a name="Bjos5"></a>
# 五、release包做了什么？
在 Rust 中，使用 cargo build 和 cargo build --release 会生成两种不同模式下的二进制文件，其中 release 模式会进行额外的编译优化。在你给出的代码中，cargo build 与 cargo build --release 的主要区别在于编译器针对代码的优化程度：

1. **编译器优化差异：** 使用 cargo build --release 会应用更多的编译器优化，这些优化可能包括函数内联、循环展开、常量折叠、无用代码消除等。这些优化可能会改变代码的执行顺序或者导致某些代码被完全优化掉。
2. **死代码消除：** 在 release 模式下，编译器会尝试消除未使用的代码或者表达式，这可能会导致一些看似没有问题的代码被编译器优化掉。
3. **内联函数：** Release 模式下编译器更倾向于内联函数，这可能会改变函数调用的方式甚至导致某些函数被内联优化。

在你的代码中，release 模式的优化可能导致以下情况：

- 如果某些代码被编译器认为是无用代码，那么这部分代码可能会被直接消除，导致你的输出结果出现问题。

总的来说，就是release包给优化了。
<a name="XZVMM"></a>
# 六、如何减少release包的优化
我们可以在`.cargo/config.toml`中配置release的优化程度。
```toml
# .cargo/config.toml
[profile.release]
opt-level = 0
```
这里设置opt-level选项，设置该项的值为0。也就是不优化。

**当设置该项后，我们的项目就可以成功输出"Hello"了**。

<a name="wxI7i"></a>
# 七、profile.release还有哪些配置项？
在 Rust 中，[profile.release] 标签可以在 .cargo/config 文件中使用，并可以包含一系列配置项，用于配置 Rust 项目在 release 模式下的构建参数。以下是一些常见的配置项：

1. **opt-level**: 控制优化级别，可选值为 0、1、2 或 3。
   - 0: 不进行优化。
   - 1: 进行基本的优化，但编译速度较快。
   - 2: 进行更多的优化，可能会花费更多时间编译。
   - 3: 进行所有可能的优化，可能显著增加编译时间。

示例：
```toml
[profile.release]
opt-level = 3
```

2. **debug**: 控制是否在 release 构建中包含调试符号。可选值为 true 或 false。
   - true: 包含调试符号，方便进行调试。
   - false: 不包含调试符号，可以减小生成的可执行文件大小。

示例：
```toml
[profile.release]
debug = false
```

3. **codegen-units**: 控制编译单元的数量，用于并行编译。可以是一个数字，例如 16 或 20。

示例：
```toml
[profile.release]
codegen-units = 16
```

4. **lto**: 控制是否启用链接时优化 (link-time optimization)。可选值为 true 或 false。

示例：
```toml
[profile.release]
lto = true
```
这些配置项可以帮助你优化和定制 Rust 项目在 release 模式下的构建行为，提高可执行文件的性能和效率。
