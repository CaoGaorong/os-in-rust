#![no_std]
#![no_main]

use core::panic::PanicInfo;

use kernel::println;

use rrt::{_start, env};

#[no_mangle]
pub extern "C" fn main() {
    println!("Hello, I'm user process");
    
    
    // 获取传过来的参数
    let arg = env::get_args();
    if arg.is_some() {
        println!("args: {}", arg.unwrap());
    } else {
        println!("no args");
    }


    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}