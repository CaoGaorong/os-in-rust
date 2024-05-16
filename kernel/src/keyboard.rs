use os_in_rust_common::racy_cell::RacyCell;

use crate::scancode::{self, ScanCodeType};

static CTRL_DOWN: RacyCell<bool> = RacyCell::new(false);
static SHIFT_DOWN: RacyCell<bool> = RacyCell::new(false);
static ALT_DOWN: RacyCell<bool> = RacyCell::new(false);
static CAPS_LOCK: RacyCell<bool> = RacyCell::new(false);
static EXT_SCAN_CODE: RacyCell<bool> = RacyCell::new(false);


pub fn process_scan_code(scan_code: u8) {
    let in_extend = unsafe { EXT_SCAN_CODE.get_mut() };
    let ctrl_down = unsafe { CTRL_DOWN.get_mut() };
    let shift_down = unsafe { SHIFT_DOWN.get_mut() };
    let alt_down = unsafe { ALT_DOWN.get_mut() };
    let caps_lock = unsafe { CAPS_LOCK.get_mut() };
    
    // 扩展标记
    if is_ext_code(scan_code) {
        *in_extend = true;
        // 等待下一个scan code
        return;
    }

    let scan_code_type = get_code_type(scan_code);
    match scan_code_type {
        // 如果是放开键
        ScanCodeType::BreakCode => {
            let control_key_option = scancode::get_control_key(scan_code, in_extend);
            if let Option::None = control_key_option {
                return;
            }
            let control_key = control_key_option.unwrap();
            match control_key {
                scancode::ControlKey::LeftShift => *shift_down = false,
                scancode::ControlKey::RightShift => *shift_down = false,
                scancode::ControlKey::LeftAlt => *alt_down = false,
                scancode::ControlKey::RightAlt => *alt_down = false,
                scancode::ControlKey::LeftCtl => *ctrl_down = false,
                scancode::ControlKey::RightCtrl => *ctrl_down = false,
                scancode::ControlKey::CapsLock => *caps_lock = false,
            }
            return;
        },
        // 按下
        ScanCodeType::MakeCode => {
            
        },
        
    }



}
