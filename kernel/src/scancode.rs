use core::ptr::null;

use os_in_rust_common::{constants, ASSERT};

/**
 * 本文件，专门对扫描码进行处理
 * 把纯数字的扫描码，转换成更加有结构性的键码信息，方便业务处理
 */

/**
 * 扫描码和通码和字符的映射，关系如下：
 * index: 通码（按下键产生的码）
 * value: 三元组(键，该键的字符，该键大写字符)
 */
static MAKE_CODE_ASCII_MAPPING: [(Key, char, char); constants::KEYBOARD_KEY_COUNT] = [
    // 0x00
    (Key::Null, '\0', '\0'),
    (Key::Esc, 0x1b as char, 0x1b as char),
    (Key::One, '1', '!'),
    (Key::Two, '2', '@'),
    (Key::Three, '3', '#'),
    (Key::Four, '4', '$'),
    (Key::Five, '5', '%'),
    (Key::Six, '6', '^'),
    (Key::Seven, '7', '&'),
    (Key::Eight, '8', '*'),
    (Key::Night, '9', '('),
    (Key::Zero, '0', ')'),
    (Key::Dash, '-', '_'),
    (Key::Equals, '=', '+'),
    // 0x0E
    (Key::Backspace, 0x8 as char, 0x8 as char),
    // 0x0F
    (Key::Tab, 0x9 as char, 0x9 as char),
    // 0x10
    (Key::Q, 'q', 'Q'),
    // 0x11
    (Key::W, 'w', 'W'),
    // 0x12
    (Key::E, 'e', 'E'),
    // 0x13
    (Key::R, 'r', 'R'),
    // 0x14
    (Key::T, 't', 'T'),
    // 0x15
    (Key::Y, 'y', 'Y'),
    // 0x16
    (Key::U, 'u', 'U'),
    // 0x17
    (Key::I, 'i', 'I'),
    // 0x18
    (Key::O, 'o', 'O'),
    // 0x19
    (Key::P, 'p', 'P'),
    // 0x1A
    (Key::LeftBracket, '[', '{'),
    // 0x1B
    (Key::RightBracket, ']', '}'),
    // 0x1C
    (Key::Enter, 0x0d as char, 0x0d as char),
    // 0x1D
    (Key::LeftCtrl, '\0', '\0'),
    // 0x1E
    (Key::A, 'a', 'A'),
    // 0x1F
    (Key::S, 's', 'S'),
    // 0x20
    (Key::D, 'd', 'D'),
    // 0x21
    (Key::F, 'f', 'F'),
    // 0x22
    (Key::G, 'g', 'G'),
    // 0x23
    (Key::H, 'h', 'H'),
    // 0x24
    (Key::J, 'j', 'J'),
    // 0x25
    (Key::K, 'k', 'K'),
    // 0x26
    (Key::L, 'l', 'L'),
    // 0x27
    (Key::Semicolon, ';', ':'),
    // 0x28
    (Key::Quote, '\'', '"'),
    // 0x29
    (Key::Tilde, '`', '~'),
    // 0x2A
    (Key::LeftShift, '\0', '\0'),
    // 0x2B
    (Key::Pipe, '\\', '|'),
    // 0x2C
    (Key::Z, 'z', 'Z'),
    // 0x2D
    (Key::X, 'x', 'X'),
    // 0x2E
    (Key::C, 'c', 'C'),
    // 0x2F
    (Key::V, 'v', 'V'),
    // 0x30
    (Key::B, 'b', 'B'),
    // 0x31
    (Key::N, 'n', 'N'),
    // 0x32
    (Key::M, 'm', 'M'),
    // 0x33
    (Key::LessThan, ',', '<'),
    // 0x34
    (Key::GraterThan, '.', '>'),
    // 0x35
    (Key::Slash, '/', '?'),
    // 0x36
    (Key::RightShift, '\0', '\0'),
    // 0x37
    (Key::Asterisk, '*', '*'),
    // 0x38
    (Key::LeftAlt, '\0', '\0'),
    // 0x39
    (Key::Space, ' ', ' '),
    // 0x3A
    (Key::CapsLock, '\0', '\0'),
];

/**
 * 目前适配的所有的键。value是这个键的通码
 */
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Key {
    Null = 0x00,
    Esc = 0x01,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Night,
    Zero,
    Dash,
    Equals,
    Backspace,
    Tab = 0x0f,
    Q,
    W,
    E,
    R,
    T,
    Y,
    U,
    I,
    O,
    P,
    LeftBracket,
    RightBracket,
    Enter,
    LeftCtrl = 0x1D,
    A = 0x1E,
    S,
    D,
    F,
    G,
    H,
    J,
    K,
    L,
    Semicolon,
    Quote,
    Tilde = 0x29,
    LeftShift = 0x2A,
    Pipe = 0x2B,
    Z = 0x2C,
    X,
    C,
    V,
    B,
    N,
    M,
    LessThan,
    GraterThan,
    Slash,
    RightShift,
    Asterisk = 0x37,
    LeftAlt = 0x38,
    Space = 0x39,
    CapsLock = 0x3A,
    RightAlt = 0xE038,
    RightCtrl = 0xE01D,
}

/**
 * 键码。把扫描码转换成这个，然后做业务处理
 */
#[derive(Debug, Clone, Copy)]
pub struct KeyCode {
    /**
     * 扫描码（完整的）
     */
    pub scan_code: u16,
    /**
     * 键的枚举
     */
    pub key: Key,
    /**
     * 扫描码类型
     */
    pub code_type: ScanCodeType,
    /**
     * 该扫描码对应的字符（ascii）
     */
    pub char: char,
    /**
     * 该键大写字符
     */
    pub char_cap: char,
}

/**
 * 扫描码的类型
 */
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ScanCodeType {
    /**
     * 通码（键按下去产生）
     */
    MakeCode,
    /**
     * 断码（键放开产生）
     */
    BreakCode,
}

impl KeyCode {
    pub const fn empty() -> Self {
        Self {
            scan_code: 0,
            key: Key::Null,
            code_type: ScanCodeType::MakeCode,
            char: '\0',
            char_cap: '\0',
        }
    }
    fn new(scan_code: u16, key: Key, code_type: ScanCodeType, char: char, char_cap: char) -> Self {
        Self {
            scan_code,
            key,
            code_type,
            char,
            char_cap,
        }
    }
    /**
     * 根据扫描码，得到一个完整的键码信息
     * 注意这里的扫描码是加上了扩展码的，16个字节
     */
    pub fn get_from_scan_code(scan_code: u16) -> Option<Self> {
        // 第8位不是0，说明是断码，否则是通码
        let code_type = if scan_code & 0x0080 != 0 {
            ScanCodeType::BreakCode
        } else {
            ScanCodeType::MakeCode
        };

        // 得到通码
        let make_code = scan_code & 0xff7f;

        if (make_code as usize) < MAKE_CODE_ASCII_MAPPING.len() {
            // 根据通码，找到这个键
            let (key, low_char, high_char) = MAKE_CODE_ASCII_MAPPING[make_code as usize];
            // 构建键码信息
            return Option::Some(KeyCode::new(scan_code, key, code_type, low_char, high_char));
        } else {
            // 特殊判断
            if make_code == Key::RightAlt as u16 {
                return Option::Some(KeyCode::new(
                    scan_code,
                    Key::RightAlt,
                    code_type,
                    '\0',
                    '\0',
                ));
            // 特殊判断
            } else if make_code == Key::RightCtrl as u16 {
                return Option::Some(KeyCode::new(
                    scan_code,
                    Key::RightCtrl,
                    code_type,
                    '\0',
                    '\0',
                ));
            }
        }

        Option::None
    }
}
