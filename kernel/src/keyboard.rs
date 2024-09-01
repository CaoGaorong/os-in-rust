use os_in_rust_common::{printkln, racy_cell::RacyCell};

use crate::{ascii::AsciiKey, blocking_queue::{ArrayBlockingQueue, BlockingQueue}, scancode::{Key, KeyCode, ScanCodeType}};



struct KeyBoard {
    /**
     * 键盘的caps_lock有没有按下去
     */
    caps_lock: bool,
    /**
     * 键盘的shift是不是处于按着
     */
    shift_down: bool,
    /**
     * 此刻键盘按下的键
     */
    key: KeyCode,
}
impl KeyBoard {
    pub const fn new() -> Self {
        Self { 
            caps_lock: false,
            shift_down: false,
            key: KeyCode::empty(),
        }
    }
    /**
     * 键盘按下了一个键
     */
    #[inline(never)]
    pub fn enter(&mut self, keycode: KeyCode) {
        // capsLock键
        if keycode.key == Key::CapsLock {
            // capsLock放下，才算锁住了大写
            if keycode.code_type == ScanCodeType::BreakCode {
                self.caps_lock = !self.caps_lock;
            }
        }
        // shift键
        if keycode.key == Key::LeftShift || keycode.key == Key::RightShift {
            if keycode.code_type == ScanCodeType::MakeCode {
                self.shift_down = true;
            } else {
                self.shift_down = false;
            }
        }
        // 赋值
        self.key = keycode;
    }

    /**
     * 获取当前键盘键入键对应的ascii码
     */
    #[inline(never)]
    pub fn get_ascii(&self) -> AsciiKey {
        if self.is_capital() {
            return self.key.char_cap;
        }
        self.key.char
    }

    /**
     * 当前键盘是否处于大写的场景下
     */
    #[inline(never)]
    fn is_capital(&self) -> bool {
        // 如果caps_lock和shift都没按，那么都是小写
        if !self.caps_lock && !self.shift_down {
            return false;
        }
        // 如果没有按caps_lock，而按了shift，那么是大写
        if !self.caps_lock && self.shift_down {
            return true;
        }
        // 如果按了caps_lock，没有按着shift，大写
        if self.caps_lock && !self.shift_down {
            return true;
        }
        // 如果按了caps_lock，并且按着shift，小写
        if self.caps_lock && self.shift_down {
            return false;
        }
        return false;
    }
}


/**
 * 扫描码合并器。
 * 把8位的扫描码，那些扩展的进行合并，得到完整的键码
 */
pub struct ScanCodeCombinator {
    /**
     * 是否收到了extend码。这样用于把多个码合并
     */
    received_ext: bool,
}

impl ScanCodeCombinator {
    pub const fn new() -> Self {
        Self {
            received_ext: false
        }
    }

    /**
     * 把扫描码合并，然后转成键码信息，然后进行处理
     * scan_code: 收到的扫描码（8位）
     * callback: 收到扫描码后的回调函数
     */
    #[inline(never)]
    pub fn do_combine(&mut self, scan_code: u8, callback: fn (Option<KeyCode>)) {
    
        // 扩展标记。已经收到了扩展码
        if scan_code == 0xe0 {
            self.received_ext = true;
            // 等待下一个scan code
            return;
        }

        let mut full_scan_code: u16 = scan_code as u16;
        // 如果本身已经收到过扩展码了，那么本次的码拼接上扩展码
        if self.received_ext {
            full_scan_code |= 0xe000;
            self.received_ext = false;
        }
    
        // 把扫描码转成键码
        let key_code = KeyCode::get_from_scan_code(full_scan_code);
        
        // 把键码，回调处理
        callback(key_code);
    }
    
}

// 设置一个全局的扫描码合并器
static COMBINATOR: RacyCell<ScanCodeCombinator> = RacyCell::new(ScanCodeCombinator::new());

// 当前系统的键盘
static KEYBOARD: RacyCell<KeyBoard> = RacyCell::new(KeyBoard::new());

/**
 * 获取当前使用的键盘
 */
fn get_keyboard() -> &'static mut KeyBoard {
    unsafe { KEYBOARD.get_mut() }
}

// 键盘键的缓冲区，利用阻塞队列
static mut BUFFER: [AsciiKey; 1000] = [AsciiKey::NUL;1000];
static KEYCODE_BLOCKING_QUEUE: RacyCell<ArrayBlockingQueue<AsciiKey>> = RacyCell::new(ArrayBlockingQueue::new(unsafe { &mut BUFFER }));


/**
 * 得到键码阻塞队列
 */
#[inline(never)]
pub fn get_keycode_queue() -> &'static mut ArrayBlockingQueue<'static, AsciiKey> {
    unsafe { KEYCODE_BLOCKING_QUEUE.get_mut() }
}

/**
 * 扫描码处理
 */
#[inline(never)]
pub fn scan_code_handler(scan_code: u8) {

    // 得到合并器
    unsafe { COMBINATOR.get_mut() }
    // 进行合并扩展码，得到合并后完整的键码
    .do_combine(scan_code, |key_code_opt| {
        if key_code_opt.is_none() {
            return;
        }
        let keycode = key_code_opt.unwrap();

        // 取出当前的键盘对象
        let keyboard = self::get_keyboard();
        // 键入一个键
        keyboard.enter(keycode);

        // 断码不放入队列
        if keycode.code_type == ScanCodeType::BreakCode {
            return;
        }
        
        // 把键入的ascii码，放入队列中
        get_keycode_queue().put(keyboard.get_ascii());
    });
}
