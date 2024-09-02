
use os_in_rust_common::{racy_cell::RacyCell, MY_PANIC};

use crate::{ascii::AsciiKey, print, println, scancode::{Key, ScanCodeType}, sys_call::{self}};

use super::{cmd::Cmd, cmd_cd, cmd_dispatcher, shell::Shell};


const PATH_LEN: usize = 100;
const INPUT_LEN: usize = 100;

/**
 * shell的工作目录
 */
static SHELL: RacyCell<Shell<PATH_LEN, INPUT_LEN>> = RacyCell::new(Shell::new([0; PATH_LEN], [0; INPUT_LEN]));

/**
 * 按住的上一个键
 */
static LAST_KEY: RacyCell<AsciiKey> = RacyCell::new(AsciiKey::NUL);

fn get_last_key() -> AsciiKey {
    *unsafe { LAST_KEY.get_mut() }
}
fn set_last_key(key: AsciiKey) {
    *unsafe { LAST_KEY.get_mut() } = key;
}

static CAPITAL: RacyCell<bool> = RacyCell::new(false);

fn get_capital() -> bool {
    *unsafe { CAPITAL.get_mut() }
}

fn set_capital(capital: bool) {
    *unsafe { CAPITAL.get_mut() } = capital;
}


#[inline(never)]
fn print_prompt(shell: &mut Shell<PATH_LEN, INPUT_LEN>)
{
    print!("[imcgr@localhost {}]$ ", shell.get_cwd());
}

/**
 * 读取line
 */
#[inline(never)]
fn read_line(shell: &mut Shell<PATH_LEN, INPUT_LEN>) -> &str {
    shell.clear_input();
    self::set_capital(false);
    loop {
        let ascii_key = sys_call::read_key();

        // 如果是回车键，直接本次命令输入结束
        if ascii_key == AsciiKey::CR {
            break;
        }

        // ctrl + l，清屏
        if self::get_last_key() == AsciiKey::DC1 && ascii_key == AsciiKey::l {
            sys_call::clear_screen();
            shell.clear_input();
            break;
        }

        // ctrl + u，清除当前行
        if self::get_last_key() == AsciiKey::DC1 && ascii_key == AsciiKey::u {
            // 清空缓冲区
            let cmd = shell.get_input();
            for _ in  0..cmd.len() {
                print!("{}", 0x8 as char);
                shell.pop_last_input();
            }
            continue;
        }
        // 最后一个key
        self::set_last_key(ascii_key);

        let key_char = ascii_key as u8 as char;

        // 如果是退格键，需要删除缓冲区里一个元素
        if ascii_key == AsciiKey::BS {
            shell.pop_last_input();
            print!("{}", key_char);
            continue;
        }

        // 控制字符，不接收
        if key_char.is_ascii_control() {
            continue;
        }
        // 其他的键，放入命令队列中
        shell.append_input(key_char);
        // 并且打印出来
        print!("{}", key_char);
    }
    shell.get_input()
}

#[inline(never)]
fn exec_cmd(shell: &mut Shell<PATH_LEN, INPUT_LEN>, buf: &mut [u8]) {
    let cmd = shell.get_cmd();
    // 如果是更换目录
    if cmd.is_some() {
        let (cmd, param) = cmd.unwrap();
        if cmd == Cmd::Cd {
            let path = cmd_cd::cd(shell.get_cwd(), param, buf);
            if path.is_none() {
                println!("cd {} error, not exist", param.unwrap());
                return;
            }
            shell.set_cwd(path.unwrap());
            return;
        }
    }
    cmd_dispatcher::dispatch_cmd(shell.get_cwd(), shell.get_input(), buf);

}


#[inline(never)]
pub fn shell_start() {
    // println!("shell start, shell:{}", "/");
    // 默认shell是根目录
    let shell = unsafe { SHELL.get_mut() };
    shell.set_cwd("/");
    let buf: &mut [u8; 100] = sys_call::malloc(100);
    loop {
        // 打印提示
        self::print_prompt(shell);
        let input = self::read_line(shell);
        if input.trim().is_empty() {
            continue;
        }
        self::exec_cmd(shell, buf);
        println!();
    }
}