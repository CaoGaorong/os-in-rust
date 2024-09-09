pub struct ExecParam<'a> {
    /**
     * 执行某个文件的绝对路径
     */
    file_path: &'a str,
    /**
     * 执行某个文件的参数
     */
    args: Option<&'a str>,
}
impl <'a> ExecParam<'a> {
    pub fn new(file_path: &'a str, args: Option<&'a str>) -> Self {
        Self {
            file_path,
            args,
        }
    }

    pub fn get_file_path(&self) -> &str {
        self.file_path
    }

    pub fn get_args(&self) -> Option<&str> {
        self.args
    }
}