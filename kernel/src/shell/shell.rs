use os_in_rust_common::{array_deque::ArrayDeque, cstr_write, cstring_utils, ASSERT, MY_PANIC};

use crate::{keyboard, scancode};


/**
 * 构造一个shell对象
 */
pub struct Shell<const PATH_LEN: usize, const CMD_LEN: usize> {
    /**
     * shell的工作目录
     */
    path: [u8; PATH_LEN],
    /**
     * shell存放命令的命令行
     */
    cmd: ArrayDeque<u8, CMD_LEN>,
}

impl <const PATH_LEN: usize, const CMD_LEN: usize> Shell<PATH_LEN, CMD_LEN> {
    pub const fn new(path: [u8; PATH_LEN], cmd: [u8; CMD_LEN]) -> Self {
        Self {
            path,
            cmd: ArrayDeque::new(cmd),
        }
    }
    /**
     * 设置shell的工作目录
     */
    #[inline(never)]
    pub fn set_cwd(&mut self, cwd: &str) {
        cstr_write!(&mut self.path, "{}", cwd);
    }

    /**
     * 获取当前shell的工作目录
     */
    #[inline(never)]
    pub fn get_cwd(&self) -> &str {
        let cwd = cstring_utils::read_from_bytes(&self.path);
        ASSERT!(cwd.is_some());
        return cwd.unwrap();
    }

    /**
     * 在当前命令里追加一个字符
     */
    #[inline(never)]
    pub fn append_cmd(&mut self, char: char) {
        let mut arr = [0; 5];
        let bytes = char.encode_utf8(&mut arr).as_bytes();
        for byte in bytes {
            self.cmd.append(*byte);
        }
    }

    /**
     * 踢出命令的最后一个字符
     */
    
    pub fn pop_last_cmd(&mut self) {
        self.cmd.pop_last();
    }

    /**
     * 获取当前的命令
     */
    #[inline(never)]
    pub fn get_cmd(&self) -> &str {
        let cmd = core::str::from_utf8(&self.cmd.get_array());
        if cmd.is_err() {
            MY_PANIC!("[sys error] failed to get cmd. cmd:{}", cmd.err().unwrap());
        }
        cmd.unwrap()
    }

    #[inline(never)]
    pub fn clear_cmd(&mut self) {
        self.cmd.clear();
    }
}