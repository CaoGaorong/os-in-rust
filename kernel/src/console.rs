use core::fmt;

use lazy_static::lazy_static;
use os_in_rust_common::{printk, printkln, racy_cell::RacyCell, vga};

use crate::mutex::Mutex;

lazy_static!{
    pub static ref DEFAULT_CONSOLE: RacyCell<Mutex<Console>> = RacyCell::new(Mutex::new(Console::new()));
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

#[inline(never)]
pub fn console_print(args: fmt::Arguments) {
    unsafe { DEFAULT_CONSOLE.get_mut().lock() }.print(args);
}

pub fn console_print_char(ch: char) {
    unsafe { DEFAULT_CONSOLE.get_mut().lock().print_char(ch) };
}

pub fn clear_row() {
    unsafe { DEFAULT_CONSOLE.get_mut().lock().clear_row() };
}

pub fn clear_all() {
    unsafe { DEFAULT_CONSOLE.get_mut().lock().clear_all() };
}


pub struct Console {}
impl Console {
    pub const fn new() -> Self {
        Self {  }
    }
    #[inline(never)]
    pub fn print(&self, args: fmt::Arguments) {
        vga::print(args);
    }
    // 输出单个字符
    pub fn print_char(&self, ch: char) {
        vga::print_char(ch);
    }
    pub fn clear_row(&self) {
        vga::clear_current_row()
    }
    pub fn clear_all(&self) {
        vga::clear_all();
    }
}