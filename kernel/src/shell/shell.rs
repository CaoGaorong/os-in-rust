use os_in_rust_common::{array_deque::ArrayDeque, cstr_write, cstring_utils, ASSERT, MY_PANIC};

use super::{cmd, shell_util};

/**
 * 构造一个shell对象
 */
pub struct Shell<const PATH_LEN: usize, const CMD_LEN: usize> {
    /**
     * shell的工作目录
     */
    cwd: [u8; PATH_LEN],
    /**
     * shell存放命令的命令行
     */
    input: ArrayDeque<u8, CMD_LEN>,
}

impl <const PATH_LEN: usize, const CMD_LEN: usize> Shell<PATH_LEN, CMD_LEN> {
    pub const fn new(cwd: [u8; PATH_LEN], input: [u8; CMD_LEN]) -> Self {
        Self {
            cwd,
            input: ArrayDeque::new(input),
        }
    }
    /**
     * 设置shell的工作目录
     */
    #[inline(never)]
    pub fn set_cwd(&mut self, cwd: &str) {
        cstr_write!(&mut self.cwd, "{}", cwd);
    }

    /**
     * 获取当前shell的工作目录
     */
    #[inline(never)]
    pub fn get_cwd(&self) -> &str {
        let cwd = cstring_utils::read_from_bytes(&self.cwd);
        ASSERT!(cwd.is_some());
        return cwd.unwrap();
    }

    /**
     * 在当前命令里追加一个字符
     */
    #[inline(never)]
    pub fn append_input(&mut self, char: char) {
        let mut arr = [0; 5];
        let bytes = char.encode_utf8(&mut arr).as_bytes();
        for byte in bytes {
            self.input.append(*byte);
        }
    }

    /**
     * 踢出命令的最后一个字符
     */
    
    pub fn pop_last_input(&mut self) {
        self.input.pop_last();
    }

    /**
     * 获取当前的命令
     */
    #[inline(never)]
    pub fn get_input(&self) -> &str {
        let cmd = core::str::from_utf8(&self.input.get_array());
        if cmd.is_err() {
            MY_PANIC!("[sys error] failed to get cmd. cmd:{}", cmd.err().unwrap());
        }
        cmd.unwrap()
    }

    #[inline(never)]
    pub fn clear_input(&mut self) {
        self.input.clear();
    }

    /**
     * 从shell的输入中，解析出命令以及该命令需要的参数
     * 例如，输入是 ls -alh -s
     * 解析出来的是 Option::Some(Cmd::Ls, "-alh -s")
     */
    #[inline(never)]
    pub fn get_cmd(&self) -> Option<(cmd::Cmd, Option<&str>)> {
        let input = self.get_input().trim();
        if input.is_empty() {
            return Option::None;
        }
        let input_split = input.split_once(" ");
        if input_split.is_none() {
            let cmd = cmd::Cmd::get_by_name(input);
            return Option::Some((cmd, Option::None));
        }
        let (cmd, argv) = input_split.unwrap();
        let cmd = cmd::Cmd::get_by_name(cmd.trim());
        let param = if argv.trim().is_empty() {Option::None} else {Option::Some(argv.trim())};
        Option::Some((cmd, param))
    }
}