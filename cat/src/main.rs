#![no_std]
#![no_main]

use core::panic::PanicInfo;

use kernel::{println, shell::shell_util, sys_call};

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

    let buff: &mut [u8; 20] = sys_call::malloc(20);
    println!("cwd:{}, input_path:{}", cwd, input_path);
    // let abs_path = shell_util::get_abs_path(cwd, input_path, buff).unwrap();
    // println!("abs path:{}", abs_path);

}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("user process panic");
    loop {}
}