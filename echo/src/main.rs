#![no_std]
#![no_main]

use core::panic::PanicInfo;

use kernel::{filesystem::{FileDescriptor, StdFileDescriptor}, print, println, shell::shell_util, sys_call};

use rrt::{_start, env};

#[inline(never)]
#[no_mangle]
pub extern "C" fn main() {
    let args = env::get_args();
    if args.is_none() || args.unwrap().trim().is_empty() {
        return;
    }
    let args = args.unwrap().trim();
    sys_call::write(FileDescriptor::new(StdFileDescriptor::StdOutputNo as usize), args.as_bytes());
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("echo process panic, error:{:?}", _info);
    loop {}
}