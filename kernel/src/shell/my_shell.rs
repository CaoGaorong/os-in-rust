
use os_in_rust_common::{racy_cell::RacyCell, MY_PANIC};

use crate::{print, println, scancode::{Key, ScanCodeType}, sys_call::{self}};

use super::{cmd::Cmd, cmd_cd::{self}, cmd_custom, cmd_dir, cmd_ls, cmd_ps, shell::Shell};


const PATH_LEN: usize = 100;
const INPUT_LEN: usize = 100;

/**
 * shell的工作目录
 */
static SHELL: RacyCell<Shell<PATH_LEN, INPUT_LEN>> = RacyCell::new(Shell::new([0; PATH_LEN], [0; INPUT_LEN]));

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
fn read_line(shell: &mut Shell<PATH_LEN, INPUT_LEN>) -> &str {
    shell.clear_input();
    loop {
        let keycode = sys_call::read_key();

        // 断码，忽略
        if keycode.code_type == ScanCodeType::BreakCode {
            set_last_key(Key::Null);
            // println!("break code ignored");
            continue;
        }

        // 如果是回车键，直接本次命令输入结束
        if keycode.key == Key::Enter {
            break;
        }

        // ctrl + l，清屏
        if self::get_last_key() == Key::LeftCtrl && keycode.key == Key::L {
            sys_call::clear_screen();
            shell.clear_input();
            break;
        }

        // ctrl + u，清除当前行
        if self::get_last_key() == Key::LeftCtrl && keycode.key == Key::U {
            // 清空缓冲区
            let cmd = shell.get_input();
            for _ in  0..cmd.len() {
                print!("{}", 0x8 as char);
                shell.pop_last_input();
            }
            continue;
        }

        // 最后一个key
        self::set_last_key(keycode.key);

        // 如果是退格键，需要删除缓冲区里一个元素
        if keycode.key == Key::Backspace {
            shell.pop_last_input();
            print!("{}", keycode.char);
            continue;
        }

        // 非数字字母，不接收
        if !keycode.char.is_ascii() {
            continue;
        }

        // 其他的键，放入命令队列中
        shell.append_input(keycode.char);
        // 并且打印出来
        print!("{}", keycode.char);
    }
    shell.get_input()
}

#[inline(never)]
fn exec_cmd(shell: &mut Shell<PATH_LEN, INPUT_LEN>, buf: &mut [u8]) {
    unsafe { buf.as_mut_ptr().write_bytes(0, buf.len()) };
    
    let cmd = shell.get_cmd();
    if cmd.is_none() {
        println!("command {} not found", shell.get_input());
        return;
    }
    let (cmd, param) = cmd.unwrap();
    match cmd { 
        Cmd::Pwd => {
            print!("{}", shell.get_cwd());
        },
        Cmd::Ps => cmd_ps::ps(),
        Cmd::Ls => {
            cmd_ls::ls(shell.get_cwd(), param, buf);
        },
        Cmd::Cd => {
            let res = cmd_cd::cd(shell.get_cwd(), param, buf);
            if res.is_some() {
                shell.set_cwd(res.unwrap());
            } else {
                println!("cd error {} not exist", param.unwrap());
            }
        },
        // 清屏
        Cmd::Clear => {
            sys_call::clear_screen();
        },
        // 创建目录
        Cmd::Mkdir => cmd_dir::mkdir(shell.get_cwd(), param, buf),
        // 删除目录
        Cmd::Rmdir => cmd_dir::rmdir(shell.get_cwd(), param, buf),
        Cmd::Custom(cmd) => {
            let fork_res = sys_call::fork();
            match fork_res {
                sys_call::ForkResult::Parent(child_id) => {
                    // 阻塞等待子进程退出
                    let wait_res = sys_call::wait();
                    if wait_res.is_none() {
                        println!("child process does not exit");
                    }
                    let (chpid, exit_status) = wait_res.unwrap();
                    println!("child process exit. fork child pid:{}, child pid:{}, child status:{:?}", child_id.get_data(), chpid.get_data(), exit_status);
                },
                sys_call::ForkResult::Child => {
                    cmd_custom::exec(shell.get_cwd(), cmd, param, buf);
                },
            }
        },
    };
}


#[inline(never)]
pub fn shell_start() {
    // 默认shell是根目录
    let shell = unsafe { SHELL.get_mut() };
    shell.set_cwd("/");
    let buf: &mut [u8; 100] = sys_call::malloc(100);
    loop {
        // 打印提示
        self::print_prompt();
        let input = self::read_line(shell);
        println!();
        if input.trim().is_empty() {
            continue;
        }
        self::exec_cmd(shell, buf);
        println!();
    }
}