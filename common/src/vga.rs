use core::fmt::{self, Write};

/**
 * 这里实现一个往0x8b000写入数据的writer
 * 关于如何在屏幕中输出字符，可以看文档<https://en.wikipedia.org/wiki/VGA_text_mode>
 *
 * 文本模式下，显存地址范围是[0xb80000, 0xc0000)。
 * 一个屏幕，宽度为80高度为25，也就是可以展示25 * 80 = 2000个字符，其中一个字符需要2个字节来展示，因此一个屏幕需要4000个占用显存的字节。
 * 我们只需要按照约定，把要写入的数据写入到显存地址，那么我们就可以在屏幕中显示出字符了。
 *
 * 其中，每个要显示的字符占显存的2个字节，每个字符显示的约定如下：
 * 0-7 bits: 字符的ASCII码
 * 8-11 bits: 字符的字体颜色
 * 12-14 bits: 字符的背景颜色
 * 15 bits: 是否闪烁
 */
use volatile::Volatile;

use crate::utils::bool_to_int;
use crate::racy_cell::RacyCell;
#[no_mangle]
pub static WRITER: RacyCell<Writer> = RacyCell::new(Writer::new(0xC00b8000, CharAttr::new(Color::White, Color::Black, false)));
// pub static WRITER: Writer = Writer::new(0xb8000, CharAttr::new(Color::White, Color::Black, false));

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[no_mangle]
pub fn print(args: fmt::Arguments) {
    unsafe{ WRITER.get_mut().write_fmt(args).unwrap()};
}

pub fn print_char(ch: char) {
    unsafe { WRITER.get_mut().write_byte(ch as u8) };
}

pub fn clear_current_row() {
    unsafe { WRITER.get_mut().clear_current_row() };
}

pub fn clear_all() {
    unsafe { WRITER.get_mut().clear_all() };
}

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        unsafe {WRITER.get_mut().write_string(s)};
        Ok(())
    }
}

/**
 * 定义字体颜色枚举
 */
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

/**
 * 每个要展示的字符有一个展示颜色属性，
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct CharAttr(u8);
impl CharAttr {
    pub const fn new(foreground: Color, background: Color, blink: bool) -> Self {
        
        let b: u8 = bool_to_int(blink) as u8;

        Self(b << 7 | ((background as u8) << 4) | (foreground as u8))
    }
}

/**
 * 要展示在缓冲区中的单个字符。2个字节
 * 低字节是字符的ASCII码
 * 高字节是字符的属性
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct SingleChar {
    char: u8,
    attr: CharAttr,
}
impl SingleChar {
    fn new(char: u8, attr: CharAttr) -> Self {
        Self { char, attr }
    }
}

/**
 * 要输出到屏幕的缓冲区。
 * 一屏幕就是80 * 25
 */
const BUFFER_WIDTH: usize = 80;
const BUFFER_HEIGHT: usize = 25;
#[repr(transparent)]
pub struct ScreenBuffer {
    buffer: [[Volatile<SingleChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/**
 * 往显存缓冲区里面写入要展示的数据
 */
pub struct Writer {
    row_pos: usize,
    /**
     * 写入的列位置
     */
    col_pos: usize,
    /**
     * 默认的颜色属性
     */
    default_attr: CharAttr,
    /**
     * 要写入的缓冲区
     */
    buffer: u32,
}

impl Writer {
    #[no_mangle]
    fn get_buffer(&self) -> &mut ScreenBuffer{
        unsafe {
            &mut *(self.buffer as *mut ScreenBuffer)
        }
    }
    /**
     * 构建writer
     * 两个参数：要写入的缓冲区地址，该缓冲区每次写入的数据默认属性
     */
    #[no_mangle]
    pub const fn new(buffer: u32, default_attr: CharAttr) -> Self {
        Self {
            row_pos: 0,
            col_pos: 0,
            default_attr,
            buffer,
        }
    }
    fn _backspace(&mut self) {
        // 如果是第一列，没法后退了
        if self.col_pos == 0 {
            let max_width = self.get_buffer().buffer[0].len() - 1;
            // 往上推一行
            self.row_pos -= 1;
            self.col_pos = max_width;
            return;
        }
        // 如果不是第一列，那么直接后退
        self.col_pos -= 1;
    }
    #[no_mangle]
    fn _new_line(&mut self) {
        let max_width = self.get_buffer().buffer[0].len() - 1;
        let max_height = self.get_buffer().buffer.len() - 1;
        // 如果没到最后一行，可以直接换行
        if self.row_pos < max_height {
            self.row_pos += 1;
            self.col_pos = 0;
            return;
        }
        let arry_buf = self.get_buffer().buffer.as_mut();
        // 如果到最后一行了，需要把第一行移出，并且整体上移1行
        for row_idx in 1..=max_height {
            for col_idx in 0..=max_width {
                let ele = arry_buf[row_idx][col_idx].read();
                arry_buf[row_idx - 1][col_idx].write(ele);
            }
        }
        self.col_pos = 0;
        // 把最后一行清空
        self._clear_row(max_height);
    }
    fn _clear_row(&mut self, row_idx: usize) {
        let buffer = self.get_buffer().buffer.as_mut();
        if buffer.is_empty() {
            return;
        }
        if row_idx >= buffer.len() {
            return;
        }
        // 清除某一行
        for col_idx in 0..buffer[0].len() {
            buffer[row_idx][col_idx].write(SingleChar::new(0, self.default_attr));
        }
    }

    /**
     * 清屏
     */
    fn _clear_all(&mut self) {
        let buffer = self.get_buffer().buffer.as_mut();
        if buffer.is_empty() {
            return;
        }
        // 清除每一行
        for i  in 0 .. self.row_pos + 1 {
            self._clear_row(i);
        }
    }

    /**
     * 光标后移
     */
    #[no_mangle]
    fn _cursor_next(&mut self) {
        let max_width = self.get_buffer().buffer[0].len() - 1;
        // 到最后一列，换行
        if self.col_pos == max_width {
            self._new_line();
            return;
        }
        self.col_pos += 1;
    }

    /**
     * 把字节数据（不解析），写入到缓冲区
     */
    fn do_write_byte(&mut self, data: u8) {
        self.get_buffer().buffer[self.row_pos][self.col_pos]
            .write(SingleChar::new(data, self.default_attr));
        self._cursor_next();
    }
    /**
     * 输出字节数据，解析
     */
    pub fn write_byte(&mut self, byte: u8) {
        if b'\n' == byte {
            self._new_line();
            return;
        }
        // 如果是backspace字符
        if 0x8 == byte {
            // 把光标后退
            self._backspace();
            // 打印一个空的，把原本那个字符覆盖
            self.do_write_byte(0);
            // 把光标后退
            self._backspace();
            return;
        }
        self.do_write_byte(byte);
    }
    
    /**
     * 输出字符串
     */
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
    }

    pub fn clear_current_row(&mut self) {
        self._clear_row(self.row_pos);
        self.col_pos = 0;
    }

    pub fn clear_all(&mut self) {
        self._clear_all();
        self.col_pos = 0;
        self.row_pos = 0;
    }
}
