use os_in_rust_common::{print, println, racy_cell::RacyCell, vga::print};

use crate::{console, console_print, scancode::{Key, KeyCode, ScanCodeType}};


static CTRL_DOWN: RacyCell<bool> = RacyCell::new(false);
static SHIFT_DOWN: RacyCell<bool> = RacyCell::new(false);
static ALT_DOWN: RacyCell<bool> = RacyCell::new(false);
static CAPS_LOCKED: RacyCell<bool> = RacyCell::new(false);

/**
 * 打印程序输出键码信息
 */
pub fn print_key_code(key_code_opt: Option<KeyCode>) {
    if let Option::None = key_code_opt {
        console_print!("invalid key");
        return;
    }
    let key_code = key_code_opt.unwrap();

    // 根据键按下还是放开，调节控制键
    match key_code.code_type {
        // 键盘按键按下
        ScanCodeType::MakeCode => {
            if key_code.key == Key::CapsLock {
                *unsafe { CAPS_LOCKED.get_mut() } = !*unsafe { CAPS_LOCKED.get_mut() };
                return;
            }
            // alt键盘按下
            if key_code.key == Key::LeftAlt || key_code.key == Key::RightAlt {
                *unsafe { ALT_DOWN.get_mut() } = true;
                return;
            }
            // shift键按下
            if key_code.key == Key::LeftShift || key_code.key == Key::RightShift {
                *unsafe { SHIFT_DOWN.get_mut() } = true;
                return;
            }
            // ctrl键按下
            if key_code.key == Key::LeftCtrl || key_code.key == Key::RightCtrl {
                *unsafe { CTRL_DOWN.get_mut() } = true;
                return;
            }
        },
        // 键盘按键松开
        ScanCodeType::BreakCode => {
            // alt键盘松开
            if key_code.key == Key::LeftAlt || key_code.key == Key::RightAlt {
                *unsafe { ALT_DOWN.get_mut() } = false;
                return;
            }
            // shift键松开
            if key_code.key == Key::LeftShift || key_code.key == Key::RightShift {
                *unsafe { SHIFT_DOWN.get_mut() } = false;
                return;
            }
            // ctrl键松开
            if key_code.key == Key::LeftCtrl || key_code.key == Key::RightCtrl {
                *unsafe { CTRL_DOWN.get_mut() } = false;
                return;
            }
            
        },
    }

    // 对于断码，普通键（非控制键）不处理
    if key_code.code_type == ScanCodeType::BreakCode {
        return;
    }

    let mut char_to_print: char = key_code.char;
    if *unsafe { SHIFT_DOWN.get_mut() } || *unsafe { CAPS_LOCKED.get_mut() } {
        char_to_print = key_code.char_cap;
    }

    // ctrl + u，清除这一行
    if *unsafe { CTRL_DOWN.get_mut() } && key_code.key == Key::U {
        console::clear_row();
        return;
    }
    // ctrl + l，清屏
    if *unsafe { CTRL_DOWN.get_mut() } && key_code.key == Key::L {
        console::clear_all();
        return;
    }
    console::console_print_char(char_to_print);
    
}