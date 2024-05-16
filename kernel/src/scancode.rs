use core::ptr::null;

use os_in_rust_common::{constants, ASSERT};

/**
 * 构建一个通码码到Ascii码的映射。
 */
static MAKE_CODE_KEY_SET: [AsciiCodeKey; constants::KEYBOARD_KEY_COUNT] = compose_key_set();
/**
 * 不可见的Ascii码
 */
pub enum AsciiControlKey {
    Null = 0x00,
    Esc = 0x1b,
    Backspace = 0x8,
    Tab = 0x9,
    Enter = 0x0d,
    Delete = 0x7F,
}


#[derive(Clone, Copy)]
pub struct AsciiCodeKey {
    /**
     * 单个字符
     */
    pub ascii_code: char,
    /**
     * 按住shift后产生的字符
     */
    pub ascii_if_shift: char,
}
impl AsciiCodeKey {
    pub fn new(ascii_code: char, ascii_if_shift: char) -> Self {
        Self {
            ascii_code, ascii_if_shift
        }
    }
}

/**
 * 扫描码的类型
 */
pub enum ScanCodeType {
    /**
     * 通码
     */
    MakeCode,
    /**
     * 断码
     */
    BreakCode
}

pub enum ControlKey {
    LeftShift,
    RightShift,
    LeftAlt,
    RightAlt,
    LeftCtl,
    RightCtrl,
    CapsLock,
}

/**
 * 根据扫描码获取到控制的key
 */
pub fn get_control_key(scan_code: u8, in_extend: bool) -> Option<ControlKey> {
    let make_code = get_make_code(scan_code);
    if 0x2a == make_code {
        Option::Some(ControlKey::LeftShift)
    }
    if 0x36 == make_code {
        Option::Some(ControlKey::RightShift)
    }
    if 0x38 == make_code {
        if in_extend {
            Option::Some(ControlKey::RightAlt)
        }
        Option::Some(ControlKey::LeftAlt)
    }
    if 0x1d == make_code {
        if in_extend {
            Option::Some(ControlKey::RightCtrl)
        }
        Option::Some(ControlKey::LeftCtl)
    }
    if 0x3a == make_code {
        Option::Some(ControlKey::CapsLock)
    }
    Option::None

}


/**
 * 根据扫描码，得到这个码的类型
 */
pub fn get_code_type(scan_code: u8) -> ScanCodeType {
    // 最高位是不是0，说明是断码
    if scan_code & 0x80 != 0 {
        ScanCodeType::BreakCode
    }
    ScanCodeType::MakeCode
}

/**
 * 根据扫描码，得到ascii字符键
 */
pub fn get_ascii(scan_code: u8) -> Option<AsciiCodeKey> {
    let make_code = get_make_code(scan_code);
    if make_code >= MAKE_CODE_KEY_SET.len() {
        Option::None
    }
    Option::Some(MAKE_CODE_KEY_SET[make_code as usize])
}


fn get_make_code(scan_code: u8) -> u8 {
    // 保留低7位。去掉最高位（第8位）
    break_code & 0b7f
}



/**
 * 是否扩展类型的扫描码
 */
pub fn is_ext_code(scan_code: u8) -> bool {
    scan_code == 0xe0
}

fn compose_key_set() -> [AsciiCodeKey; constants::KEYBOARD_KEY_COUNT] {
    let key_set = [AsciiCodeKey::new(AsciiControlKey::Null, constants::KEYBOARD_KEY_COUNT)];
    key_set[0x00] = AsciiCodeKey::new(AsciiControlKey::Null, AsciiControlKey::Null);
    key_set[0x01] = AsciiCodeKey::new(AsciiControlKey::Esc, AsciiControlKey::Esc);
    key_set[0x02] = AsciiCodeKey::new('1', '!');
    key_set[0x03] = AsciiCodeKey::new('2', '@');
    key_set[0x04] = AsciiCodeKey::new('3', '#');
    key_set[0x05] = AsciiCodeKey::new('4', '$');
    key_set[0x06] = AsciiCodeKey::new('5', '%');
    key_set[0x07] = AsciiCodeKey::new('6', '^');
    key_set[0x08] = AsciiCodeKey::new('7', '&');
    key_set[0x09] = AsciiCodeKey::new('8', '*');
    key_set[0x0a] = AsciiCodeKey::new('9', '(');
    key_set[0x0b] = AsciiCodeKey::new('0', ')');
    key_set[0x0c] = AsciiCodeKey::new('-', '_');
    key_set[0x0d] = AsciiCodeKey::new('=', '+');
    key_set[0x0e] = AsciiCodeKey::new(AsciiControlKey::Backspace, AsciiControlKey::Backspace);
    key_set[0x0f] = AsciiCodeKey::new(AsciiControlKey::Tab, AsciiControlKey::Tab);
    key_set[0x10] = AsciiCodeKey::new('q', 'Q');
    key_set[0x11] = AsciiCodeKey::new('w', 'W');
    key_set[0x12] = AsciiCodeKey::new('e', 'E');
    key_set[0x13] = AsciiCodeKey::new('r', 'R');
    key_set[0x14] = AsciiCodeKey::new('t', 'T');
    key_set[0x15] = AsciiCodeKey::new('y', 'Y');
    key_set[0x16] = AsciiCodeKey::new('u', 'U');
    key_set[0x17] = AsciiCodeKey::new('i', 'I');
    key_set[0x18] = AsciiCodeKey::new('o', 'O');
    key_set[0x19] = AsciiCodeKey::new('p', 'P');
    key_set[0x1a] = AsciiCodeKey::new('[', '{');
    key_set[0x1b] = AsciiCodeKey::new(']', '}');
    key_set[0x1c] = AsciiCodeKey::new(AsciiControlKey::Enter, AsciiControlKey::Enter);
    key_set[0x1d] = AsciiCodeKey::new(AsciiControlKey::Null, AsciiControlKey::Null);
    key_set[0x1e] = AsciiCodeKey::new('a', 'A');
    key_set[0x1f] = AsciiCodeKey::new('s', 'S');
    key_set[0x20] = AsciiCodeKey::new('d', 'D');
    key_set[0x21] = AsciiCodeKey::new('f', 'F');
    key_set[0x22] = AsciiCodeKey::new('g', 'G');
    key_set[0x23] = AsciiCodeKey::new('h', 'H');
    key_set[0x24] = AsciiCodeKey::new('j', 'J');
    key_set[0x25] = AsciiCodeKey::new('k', 'K');
    key_set[0x26] = AsciiCodeKey::new('l', 'L');
    key_set[0x27] = AsciiCodeKey::new(';', ':');
    key_set[0x28] = AsciiCodeKey::new('\'', '"');
    key_set[0x29] = AsciiCodeKey::new('`', '~');
    key_set[0x2a] = AsciiCodeKey::new(AsciiControlKey::Null, AsciiControlKey::Null);
    key_set[0x2b] = AsciiCodeKey::new('\\', '|');
    key_set[0x2c] = AsciiCodeKey::new('z', 'Z');
    key_set[0x2d] = AsciiCodeKey::new('x', 'X');
    key_set[0x2e] = AsciiCodeKey::new('c', 'C');
    key_set[0x2f] = AsciiCodeKey::new('v', 'V');
    key_set[0x30] = AsciiCodeKey::new('b', 'B');
    key_set[0x31] = AsciiCodeKey::new('n', 'N');
    key_set[0x32] = AsciiCodeKey::new('m', 'M');
    key_set[0x33] = AsciiCodeKey::new(',', '<');
    key_set[0x34] = AsciiCodeKey::new('.', '>');
    key_set[0x35] = AsciiCodeKey::new('/', '?');
    key_set[0x36] = AsciiCodeKey::new(AsciiControlKey::Null, AsciiControlKey::Null);
    key_set[0x37] = AsciiCodeKey::new(AsciiControlKey::Null, AsciiControlKey::Null);
    key_set[0x38] = AsciiCodeKey::new(AsciiControlKey::Null, AsciiControlKey::Null);
    key_set[0x39] = AsciiCodeKey::new(' ', ' ');
    key_set[0x3a] = AsciiCodeKey::new(AsciiControlKey::Null, AsciiControlKey::Null);

    key_set
}
