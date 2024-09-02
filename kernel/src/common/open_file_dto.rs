
#[derive(Debug)]
pub struct OpenFileDto<'a> {
    pub file_path: &'a str,
    pub append: bool,
}

impl <'a> OpenFileDto<'a> {
    #[inline(never)]
    pub fn new(path: &'a str, append: bool) -> Self {
        Self { file_path: path, append: append }
    }
}