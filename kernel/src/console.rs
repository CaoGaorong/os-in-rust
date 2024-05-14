use core::fmt;

use lazy_static::lazy_static;
use os_in_rust_common::{print, println, racy_cell::RacyCell, vga};

use crate::mutex::Mutex;

lazy_static!{
    pub static ref DEFAULT_CONSOLE: RacyCell<Mutex<Console>> = RacyCell::new(Mutex::new(Console::new()));
    pub static ref INIT:() = unsafe { DEFAULT_CONSOLE.get_mut().init()};
}


#[macro_export]
macro_rules! console_print {
    ($($arg:tt)*) => ($crate::console::console_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! console_println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::console_print!("{}\n", format_args!($($arg)*)));
}

pub fn console_print(args: fmt::Arguments) {
    unsafe { DEFAULT_CONSOLE.get_mut().lock() }.print(args);
}



pub struct Console {}
impl Console {
    pub const fn new() -> Self {
        Self {  }
    }
    pub fn print(&self, args: fmt::Arguments) {
        let i = INIT;
        vga::print(args);
    }
}