use crate::filesystem::{self, DirError};

#[inline(never)]
pub fn cd(dir_path: &str) -> Result<(), DirError> {
    let mut dir = filesystem::read_dir(dir_path)?;
    Result::Ok(())
}