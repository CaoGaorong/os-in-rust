use core::fmt;
use core::fmt::Write;
use core::ops::{Add, Div, Sub};
use crate::{ASSERT, printkln};

pub const fn bool_to_int(b: bool) -> u32 {
    if b {
        1
    } else {
        0
    }
}

pub const fn bool_to_u8 (b: bool) -> u8 {
    if b {
        1
    } else {
        0
    }
}

/**
 * 两个数相除，线上取整
 */
pub fn div_ceil<T: Div + Add + Sub + Into<f64>>(num1: T, num2:T) -> f64 {
    let num1_f64:f64 = num1.into();
    let num2_f64:f64 = num2.into();
    let one_f64:f64 = 1 as f64;
    (num1_f64 + num2_f64 - one_f64) / num2_f64
}

/**
 * 计算某个结构体的成员变量所在结构体的偏移量。比如：
 * struct MyStruct  {
 *     id: u8,
 *     age: u32,
 *     sex: u8
 * }
 * fn main() {
 *     let offset = offset!(MyStruct, sex);
 *     println("offset:{}", offset);
 * }
 * 
 * 注意rust的字节默认对齐
 */
#[macro_export]
macro_rules! offset {
    ($struct_type:ty, $member:ident) => {
        unsafe { 
            &(*(0 as *const $struct_type)).$member as *const _ as usize
        }
    };
}

/**
 * 已知某个结构体和成员以及该成员的偏移量，得到该结构体的指针
 * fn main() {
 * 
 *   let my_struct = MyStruct {
 *       id: 1,
 *       age: 20,
 *       sex: 1,
 *   };
 * 
 *   let s = elem2entry!(MyStruct, age, &my_struct.age as *const u32 as usize);
 *   let generated_struct = unsafe { &mut *s };
 * 
 *   assert_eq!(&my_struct as *const _ as u32, generated_struct as *const _ as u32);
 * }
 * 
 */
#[macro_export]
macro_rules! elem2entry {
    ($struct_type:ty, $struct_member_name:ident, $elem_ptr:expr) => {
        {
            let offset = $crate::offset!($struct_type, $struct_member_name);
            ($elem_ptr as usize - offset) as *mut $struct_type
        }
    };
}


/**
 * 把一个字符串（格式化）写入到缓冲区中
 * 例如：
 *  /**定义一个缓冲区 **/
 *  let mut buf = [u8; 20];
 *
 *  /** 把字符串格式化后，写入到缓冲区**/
 *  sprintf(&mut buf, "Name: {}", "Jackson");
 *
 *  /**从缓冲区中取出字符串**/
 *  let name = core::str::from_utf8(&buf).unwrap();
 * 
 */
#[macro_export]
macro_rules! sprintf {
    ($buf:expr, $($arg:tt)*) => ($crate::utils::sprintf_fn($buf, format_args!($($arg)*)));
}

#[no_mangle]
pub fn sprintf_fn<'a>(buf: &'a mut [u8], args: fmt::Arguments) {
    let mut buffer = BufferWriter::new(buf);
    let res = buffer.write_fmt(args);
    ASSERT!(res.is_ok());
}

struct BufferWriter<'a> {
    buffer: &'a mut [u8],
}
impl <'a>BufferWriter<'a> {
    pub fn new(buf: &'a mut [u8]) -> Self {
        Self {
            buffer: buf,
        }
    }
}
impl Write for BufferWriter<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let b = &mut self.buffer[0 .. s.len()];
        b.copy_from_slice(s.as_bytes());
        Result::Ok(())
    }
}

