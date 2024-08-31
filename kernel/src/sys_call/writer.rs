use core::fmt::{self, Write};

use os_in_rust_common::racy_cell::RacyCell;

use crate::filesystem::{FileDescriptor, StdFileDescriptor};

use super::sys_call_proxy;


/**
 * 一个控制台writer，专门写入到控制台
 */
static CONSOLE_WRITER: RacyCell<FileWriter> = RacyCell::new(FileWriter::new(FileDescriptor::new(StdFileDescriptor::StdOutputNo as usize)));

/**
 * 系统调用 print!
 */
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::sys_call::sys_print(format_args!($($arg)*)));
}

/**
 * 系统调用 println!
 */
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[no_mangle]
pub fn sys_print(args: fmt::Arguments) {
    let writer = unsafe { CONSOLE_WRITER.get_mut() };
    writer.write_fmt(args).unwrap();
}

pub struct FileWriter {
    fd: FileDescriptor,
}

impl FileWriter {
    pub const fn new(fd: FileDescriptor) -> Self {
        Self {
            fd,
        }
    }
}


impl Write for FileWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        sys_call_proxy::write(self.fd, s.as_bytes());
        Result::Ok(())
    }
}