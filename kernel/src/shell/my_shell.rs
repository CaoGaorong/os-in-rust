
use os_in_rust_common::{printkln, racy_cell::RacyCell, vga::print};

use crate::{blocking_queue::BlockingQueue, keyboard, print, println, scancode::{Key, ScanCodeType}, sys_call::sys_call_proxy};

use super::shell::Shell;


const PATH_LEN: usize = 20;
const CMD_LEN: usize = 100;

/**
 * shell的工作目录
 */
static SHELL: RacyCell<Shell<PATH_LEN, CMD_LEN>> = RacyCell::new(Shell::new([0; PATH_LEN], [0; CMD_LEN]));

/**
 * 按住的上一个键
 */
static LAST_KEY: RacyCell<Key> = RacyCell::new(Key::Null);

fn get_last_key() -> Key {
    *unsafe { LAST_KEY.get_mut() }
}
fn set_last_key(key: Key) {
    *unsafe { LAST_KEY.get_mut() } = key;
}


#[inline(never)]
fn print_prompt()
{
    let shell = unsafe { SHELL.get_mut() };
    print!("[imcgr@localhost {}]$ ", shell.get_cwd());
}

/**
 * 读取line
 */
#[inline(never)]
fn read_line(shell: &mut Shell<PATH_LEN, CMD_LEN>) {
    shell.clear_cmd();
    let keycode_queue = keyboard::get_keycode_queue();
    while let Option::Some(keycode) = keycode_queue.take() {

        // 断码，忽略
        if keycode.code_type == ScanCodeType::BreakCode {
            set_last_key(Key::Null);
            continue;
        }

        // 如果是回车键，直接本次命令输入结束
        if keycode.key == Key::Enter {
            return;
        }

        // ctrl + l，清屏
        if self::get_last_key() == Key::LeftCtrl && keycode.key == Key::L {
            sys_call_proxy::clear_screen();
            return;
        }

        // ctrl + u，清除当前行
        if self::get_last_key() == Key::LeftCtrl && keycode.key == Key::U {
            // 清空缓冲区
            let cmd = shell.get_cmd();
            for _ in  0..cmd.len() {
                print!("{}", 0x8 as char);
                shell.pop_last_cmd();
            }
            continue;
        }

        // 最后一个key
        self::set_last_key(keycode.key);

        // 如果是退格键，需要删除缓冲区里一个元素
        if keycode.key == Key::Backspace {
            shell.pop_last_cmd();
            print!("{}", keycode.char);
            continue;
        }

        // 非数字字母，不接收
        if !keycode.char.is_ascii() {
            continue;
        }

        // 其他的键，放入命令队列中
        shell.append_cmd(keycode.char);
        // 并且打印出来
        print!("{}", keycode.char);
    }
}



#[inline(never)]
pub fn shell_start() {
    // 默认shell是根目录
    let shell = unsafe { SHELL.get_mut() };
    shell.set_cwd("/");
    loop {
        // 打印提示
        self::print_prompt();
        self::read_line(shell);
        println!();
    }
}