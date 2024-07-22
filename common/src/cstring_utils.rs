use core::{ffi::CStr, fmt::{self, Write}};

use crate::{ASSERT, MY_PANIC};


/**
 * 把一个字符串（以C字符串的格式）写入到缓冲区中
 * 例如：
 *  /**定义一个缓冲区 **/
 *  let mut buf = [u8; 20];
 *
 *  /** 把字符串格式化后，写入到缓冲区**/
 *  sprintf(&mut buf, "Name: {}", "Jackson");
 */
#[macro_export]
macro_rules! cstr_write {
    ($buf:expr, $($arg:tt)*) => ($crate::cstring_utils::sprintf_fn($buf, format_args!($($arg)*)));
}

#[no_mangle]
#[inline(never)]
pub fn sprintf_fn<'a>(buf: &'a mut [u8], args: fmt::Arguments) {
    // 封装成BufferContainer
    let mut buffer = BufferContainer::new(buf);
    // 把格式化的字符串写入到 BufferContainer
    let res = buffer.write_fmt(args);
    // 在buf结尾添加\0
    buffer.buffer[buffer.cur_idx] = b'\0';
    ASSERT!(res.is_ok());
}

/**
 * buffer的容器。一方面实现Write接口。另一方面记录下标
 */
struct BufferContainer<'a> {
    buffer: &'a mut [u8],
    cur_idx: usize,
}
impl <'a>BufferContainer<'a> {
    #[inline(never)]
    pub fn new(buf: &'a mut [u8]) -> Self {
        Self {
            buffer: buf,
            cur_idx: 0,
        }
    }
}
impl Write for BufferContainer<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let end = self.cur_idx + s.len();
        if self.cur_idx >= self.buffer.len() || end >= self.buffer.len() {
            MY_PANIC!("string({}) length exceed buffer capacity{}", s, self.buffer.len());
        }
        let b = &mut self.buffer[self.cur_idx .. end];
        b.copy_from_slice(s.as_bytes());
        self.cur_idx += s.len();
        Result::Ok(())
    }
}

/**
 * 从一个C格式的u8数组中，读取出正确的&str
 */
pub fn read_from_bytes(buf: &[u8]) -> Option<&str> {
    // 以C字符串格式读取，忽略结尾的\0
    let cstr = CStr::from_bytes_until_nul(buf);
    if cstr.is_err() {
        return Option::None;
    }
    let cstr = cstr.unwrap();
    let str = cstr.to_str();
    if str.is_err() {
        return Option::None;
    }
    return Option::Some(str.unwrap());
}
