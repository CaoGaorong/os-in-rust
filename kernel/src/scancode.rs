use core::ptr::null;

use os_in_rust_common::{constants, ASSERT};

use crate::ascii::AsciiKey;

/**
 * 本文件，专门对扫描码进行处理
 * 把纯数字的扫描码，转换成更加有结构性的键码信息，方便业务处理
 */

/**
 * 扫描码和通码和字符的映射，关系如下：
 * index: 通码（按下键产生的码）
 * value: 三元组(键，该键的字符，该键大写字符)
 */
static MAKE_CODE_ASCII_MAPPING: [(Key, AsciiKey, AsciiKey); constants::KEYBOARD_KEY_COUNT] = [
    // 0x00
    (Key::Null, AsciiKey::NUL, AsciiKey::NUL),
    (Key::Esc, AsciiKey::ESC, AsciiKey::ESC),
    (Key::One, AsciiKey::ONE, AsciiKey::EXCLAMATION),
    (Key::Two, AsciiKey::TWO, AsciiKey::AT),
    (Key::Three, AsciiKey::THREE, AsciiKey::HASH),
    (Key::Four, AsciiKey::FOUR, AsciiKey::DOLLAR),
    (Key::Five, AsciiKey::FIVE, AsciiKey::PERCENT),
    (Key::Six, AsciiKey::SIX, AsciiKey::CARET),
    (Key::Seven, AsciiKey::SEVEN, AsciiKey::AMPERSAND),
    (Key::Eight, AsciiKey::EIGHT, AsciiKey::ASTERISK),
    (Key::Night, AsciiKey::NINE, AsciiKey::LPAREN),
    (Key::Zero, AsciiKey::ZERO, AsciiKey::RPAREN),
    (Key::Dash, AsciiKey::MINUS, AsciiKey::UNDERSCORE),
    (Key::Equals, AsciiKey::EQUALS, AsciiKey::PLUS),
    // 0x0E
    (Key::Backspace, AsciiKey::BS, AsciiKey::BS),
    // 0x0F
    (Key::Tab, AsciiKey::TAB, AsciiKey::TAB),
    // 0x10
    (Key::Q, AsciiKey::q, AsciiKey::Q),
    // 0x11
    (Key::W, AsciiKey::w, AsciiKey::W),
    // 0x12
    (Key::E, AsciiKey::e, AsciiKey::W),
    // 0x13
    (Key::R, AsciiKey::r, AsciiKey::R),
    // 0x14
    (Key::T, AsciiKey::t, AsciiKey::T),
    // 0x15
    (Key::Y, AsciiKey::y, AsciiKey::Y),
    // 0x16
    (Key::U, AsciiKey::u, AsciiKey::U),
    // 0x17
    (Key::I, AsciiKey::i, AsciiKey::I),
    // 0x18
    (Key::O, AsciiKey::o, AsciiKey::O),
    // 0x19
    (Key::P, AsciiKey::p, AsciiKey::P),
    // 0x1A
    (Key::LeftBracket, AsciiKey::LBRACKET, AsciiKey::LBRACE),
    // 0x1B
    (Key::RightBracket, AsciiKey::RBRACKET, AsciiKey::RBRACE),
    // 0x1C
    (Key::Enter, AsciiKey::CR, AsciiKey::CR),
    // 0x1D
    (Key::LeftCtrl, AsciiKey::DC1, AsciiKey::DC1),
    // 0x1E
    (Key::A, AsciiKey::a, AsciiKey::A),
    // 0x1F
    (Key::S, AsciiKey::s, AsciiKey::S),
    // 0x20
    (Key::D, AsciiKey::d, AsciiKey::D),
    // 0x21
    (Key::F, AsciiKey::f, AsciiKey::F),
    // 0x22
    (Key::G, AsciiKey::g, AsciiKey::G),
    // 0x23
    (Key::H, AsciiKey::h, AsciiKey::H),
    // 0x24
    (Key::J, AsciiKey::j, AsciiKey::J),
    // 0x25
    (Key::K, AsciiKey::k, AsciiKey::K),
    // 0x26
    (Key::L, AsciiKey::l, AsciiKey::L),
    // 0x27
    (Key::Semicolon, AsciiKey::SEMICOLON, AsciiKey::COLON),
    // 0x28
    (Key::Quote, AsciiKey::APOSTROPHE, AsciiKey::QUOTE),
    // 0x29
    (Key::Tilde, AsciiKey::GRAVE, AsciiKey::TILDE),
    // 0x2A
    (Key::LeftShift, AsciiKey::SI, AsciiKey::SI),
    // 0x2B
    (Key::Pipe, AsciiKey::BACKSLASH, AsciiKey::PIPE),
    // 0x2C
    (Key::Z, AsciiKey::z, AsciiKey::Z),
    // 0x2D
    (Key::X, AsciiKey::x, AsciiKey::X),
    // 0x2E
    (Key::C, AsciiKey::c, AsciiKey::C),
    // 0x2F
    (Key::V, AsciiKey::v, AsciiKey::V),
    // 0x30
    (Key::B, AsciiKey::b, AsciiKey::B),
    // 0x31
    (Key::N, AsciiKey::n, AsciiKey::N),
    // 0x32
    (Key::M, AsciiKey::m, AsciiKey::M),
    // 0x33
    (Key::LessThan, AsciiKey::COMMA, AsciiKey::LESS),
    // 0x34
    (Key::GraterThan, AsciiKey::PERIOD, AsciiKey::GREATER),
    // 0x35
    (Key::Slash, AsciiKey::SLASH, AsciiKey::QUESTION),
    // 0x36
    (Key::RightShift, AsciiKey::SO, AsciiKey::SO),
    // 0x37
    (Key::Asterisk, AsciiKey::ASTERISK, AsciiKey::ASTERISK),
    // 0x38
    (Key::LeftAlt, AsciiKey::DC2, AsciiKey::DC2),
    // 0x39
    (Key::Space, AsciiKey::SPACE, AsciiKey::SPACE),
    // 0x3A
    (Key::CapsLock, AsciiKey::DC4, AsciiKey::DC4),
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
    pub char: AsciiKey,
    /**
     * 该键大写字符
     */
    pub char_cap: AsciiKey,
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
    #[inline(never)]
    pub const fn empty() -> Self {
        Self {
            scan_code: 0,
            key: Key::Null,
            code_type: ScanCodeType::MakeCode,
            char: AsciiKey::NUL,
            char_cap: AsciiKey::NUL,
        }
    }
    #[inline(never)]
    fn new(scan_code: u16, key: Key, code_type: ScanCodeType, char: AsciiKey, char_cap: AsciiKey) -> Self {
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
                    AsciiKey::DC2,
                    AsciiKey::DC2,
                ));
            // 特殊判断
            } else if make_code == Key::RightCtrl as u16 {
                return Option::Some(KeyCode::new(
                    scan_code,
                    Key::RightCtrl,
                    code_type,
                    AsciiKey::DC1,
                    AsciiKey::DC1,
                ));
            }
        }

        Option::None
    }
}
