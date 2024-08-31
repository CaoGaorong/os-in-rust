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
static MAKE_CODE_ASCII_MAPPING: [(Key, Option<char>, Option<char>); constants::KEYBOARD_KEY_COUNT] = [
    // 0x00
    (Key::Null, Option::None, Option::None),
    (Key::Esc, Option::Some(0x1b as char), Option::Some(0x1b as char)),
    (Key::One, Option::Some('1'), Option::Some('!')),
    (Key::Two, Option::Some('2'), Option::Some('@')),
    (Key::Three, Option::Some('3'), Option::Some('#')),
    (Key::Four, Option::Some('4'), Option::Some('$')),
    (Key::Five, Option::Some('5'), Option::Some('%')),
    (Key::Six, Option::Some('6'), Option::Some('^')),
    (Key::Seven, Option::Some('7'), Option::Some('&')),
    (Key::Eight, Option::Some('8'), Option::Some('*')),
    (Key::Night, Option::Some('9'), Option::Some('(')),
    (Key::Zero, Option::Some('0'), Option::Some(')')),
    (Key::Dash, Option::Some('-'), Option::Some('_')),
    (Key::Equals, Option::Some('='), Option::Some('+')),
    // 0x0E
    (Key::Backspace, Option::Some(0x8 as char), Option::Some(0x8 as char)),
    // 0x0F
    (Key::Tab, Option::Some(0x9 as char), Option::Some(0x9 as char)),
    // 0x10
    (Key::Q, Option::Some('q'), Option::Some('Q')),
    // 0x11
    (Key::W, Option::Some('w'), Option::Some('W')),
    // 0x12
    (Key::E, Option::Some('e'), Option::Some('E')),
    // 0x13
    (Key::R, Option::Some('r'), Option::Some('R')),
    // 0x14
    (Key::T, Option::Some('t'), Option::Some('T')),
    // 0x15
    (Key::Y, Option::Some('y'), Option::Some('Y')),
    // 0x16
    (Key::U, Option::Some('u'), Option::Some('U')),
    // 0x17
    (Key::I, Option::Some('i'), Option::Some('I')),
    // 0x18
    (Key::O, Option::Some('o'), Option::Some('O')),
    // 0x19
    (Key::P, Option::Some('p'), Option::Some('P')),
    // 0x1A
    (Key::LeftBracket, Option::Some('['), Option::Some('{')),
    // 0x1B
    (Key::RightBracket, Option::Some(']'), Option::Some('}')),
    // 0x1C
    (Key::Enter, Option::Some(0x0d as char), Option::Some(0x0d as char)),
    // 0x1D
    (Key::LeftCtrl, Option::None, Option::Some(0x0d as char)),
    // 0x1E
    (Key::A, Option::Some('a'), Option::Some('A')),
    // 0x1F
    (Key::S, Option::Some('s'), Option::Some('S')),
    // 0x20
    (Key::D, Option::Some('d'), Option::Some('D')),
    // 0x21
    (Key::F, Option::Some('f'), Option::Some('F')),
    // 0x22
    (Key::G, Option::Some('g'), Option::Some('G')),
    // 0x23
    (Key::H, Option::Some('h'), Option::Some('H')),
    // 0x24
    (Key::J, Option::Some('j'), Option::Some('J')),
    // 0x25
    (Key::K, Option::Some('k'), Option::Some('K')),
    // 0x26
    (Key::L, Option::Some('l'), Option::Some('L')),
    // 0x27
    (Key::Semicolon, Option::Some(';'), Option::Some(':')),
    // 0x28
    (Key::Quote, Option::Some('\''), Option::Some('"')),
    // 0x29
    (Key::Tilde, Option::Some('`'), Option::Some('~')),
    // 0x2A
    (Key::LeftShift, Option::None, Option::None),
    // 0x2B
    (Key::Pipe, Option::Some('\\'), Option::Some('|')),
    // 0x2C
    (Key::Z, Option::Some('z'), Option::Some('Z')),
    // 0x2D
    (Key::X, Option::Some('x'), Option::Some('X')),
    // 0x2E
    (Key::C, Option::Some('c'), Option::Some('C')),
    // 0x2F
    (Key::V, Option::Some('v'), Option::Some('V')),
    // 0x30
    (Key::B, Option::Some('b'), Option::Some('B')),
    // 0x31
    (Key::N, Option::Some('n'), Option::Some('N')),
    // 0x32
    (Key::M, Option::Some('m'), Option::Some('M')),
    // 0x33
    (Key::LessThan, Option::Some(','), Option::Some('<')),
    // 0x34
    (Key::GraterThan, Option::Some('.'), Option::Some('>')),
    // 0x35
    (Key::Slash, Option::Some('/'), Option::Some('?')),
    // 0x36
    (Key::RightShift, Option::None, Option::None),
    // 0x37
    (Key::Asterisk, Option::Some('*'), Option::Some('*')),
    // 0x38
    (Key::LeftAlt, Option::None, Option::None),
    // 0x39
    (Key::Space, Option::Some(' '), Option::Some(' ')),
    // 0x3A
    (Key::CapsLock, Option::None, Option::None),
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
    pub char: Option<char>,
    /**
     * 该键大写字符
     */
    pub char_cap: Option<char>,
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
            char: Option::None,
            char_cap: Option::None,
        }
    }
    fn new(scan_code: u16, key: Key, code_type: ScanCodeType, char: Option<char>, char_cap: Option<char>) -> Self {
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
    #[inline(never)]
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
                    Option::None,
                    Option::None,
                ));
            // 特殊判断
            } else if make_code == Key::RightCtrl as u16 {
                return Option::Some(KeyCode::new(
                    scan_code,
                    Key::RightCtrl,
                    code_type,
                    Option::None,
                    Option::None,
                ));
            }
        }

        Option::None
    }
}
