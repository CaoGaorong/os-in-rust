#![no_std]
#![no_main]

use core::panic::PanicInfo;

use kernel::{ascii::AsciiKey, filesystem::{FileDescriptor, StdFileDescriptor}, print, println, scancode::{Key, KeyCode, ScanCodeType}, shell::{shell_util, Shell}, sys_call};

use os_in_rust_common::{constants::KERNEL_ADDR, queue::{ArrayQueue, Queue}, vga::print};
use rrt::{_start, env};

#[inline(never)]
#[no_mangle]
pub extern "C" fn main() {
    let args = env::get_args();
    if args.is_none() || args.unwrap().trim().is_empty() {
        println!("please input string need to grep");
        return;
    }
    // 要搜索的字符串
    let grep_str = args.unwrap().trim();
    
    let input_shell: &mut Shell<20, 1000> = sys_call::malloc(size_of::<Shell<20, 1000>>());

    let mut last_key = AsciiKey::NUL;
    let mut capital = false;
    loop {

        // 从键盘中读取一个键
        let key = sys_call::read_key();
        // 读取完毕了
        if key == AsciiKey::NUL {
            break;
        }
        // 如果是contrl + c，那么就停止
        if last_key == AsciiKey::DC1 && key == AsciiKey::c {
            break;
        }
        // 如果是回车键，那么就要处理过滤了
        if key == AsciiKey::CR || key == AsciiKey::LF {
            let input = input_shell.get_input();
            // 如果输入的一行，包含我们要搜索的字符串，那么就输出
            if input.contains(grep_str) {
                println!("{}", input);
            }
            // 清除缓冲区
            input_shell.clear_input();
            continue;
        }
        last_key = key;

        // 转成字符
        let key_char = key as u8 as char;
        
        // 没有字符，过滤
        if !key_char.is_ascii() {
            continue;
        }

        assert!(input_shell.get_input().len() < 1000);
        input_shell.append_input(key_char);
    }
    println!();
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("user process panic, error:{:?}", _info);
    loop {}
}