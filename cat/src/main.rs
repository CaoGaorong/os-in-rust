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
        println!("please input file path");
        return;
    }
    let input_path = args.unwrap().trim();
    let buff: &mut [u8; 20] = sys_call::malloc(20);
    let cwd = sys_call::get_cwd(buff);

    let buff: &mut [u8; 20] = sys_call::malloc(100);
    let abs_path = shell_util::get_abs_path(cwd, input_path, buff).unwrap();

    let file = sys_call::File::open(abs_path);
    
    if file.is_err() {
        println!("failed to cat, file:{} {:?}", abs_path, file.unwrap_err());
        return;
    }
    let file = file.unwrap();
    loop {
        // 清空缓冲区
        unsafe { buff.as_mut_ptr().write_bytes(0, buff.len()) };

        // read file data from file and to buffer
        let read_bytes = file.read(buff);
        if read_bytes == 0 {
            break;
        }
        // convert byte buff to string
        let s = core::str::from_utf8(&buff[..read_bytes]);
        if s.is_err() {
            println!("error:{:?}", s.unwrap_err());
            break;
        }
        // sys_call::write(FileDescriptor::new(StdFileDescriptor::StdOutputNo as usize), buff);
        print!("{}", s.unwrap())
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("user process panic, error:{:?}", _info);
    loop {}
}