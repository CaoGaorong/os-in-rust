use os_in_rust_common::{printkln, racy_cell::RacyCell};

use crate::{blocking_queue::{ArrayBlockingQueue, BlockingQueue}, scancode::KeyCode};

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

// 键盘键的缓冲区，利用阻塞队列
static mut BUFFER: [Option<KeyCode>; 1000] = [Option::None;1000];
static KEYCODE_BLOCKING_QUEUE: RacyCell<ArrayBlockingQueue<Option<KeyCode>>> = RacyCell::new(ArrayBlockingQueue::new(unsafe { &mut BUFFER }));


/**
 * 得到键码阻塞队列
 */
#[inline(never)]
pub fn get_keycode_queue() -> &'static mut ArrayBlockingQueue<'static, Option<KeyCode>> {
    unsafe { KEYCODE_BLOCKING_QUEUE.get_mut() }
}

/**
 * 扫描码处理
 */
pub fn scan_code_handler(scan_code: u8) {

    // 得到合并器
    unsafe { COMBINATOR.get_mut() }
    // 进行合并扩展码，得到合并后完整的键码
    .do_combine(scan_code, |key_code_opt| {
        // 取得键盘队列，放一个元素
        get_keycode_queue().put(key_code_opt);
    });
}
