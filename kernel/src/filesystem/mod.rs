mod constant;
pub mod superblock;
pub mod inode;
pub mod dir;
pub mod fs;
pub mod file;
pub mod file_descriptor;
pub mod global_file_table;
pub mod filesystem;

pub use fs::init;
pub use fs::install_filesystem;
pub use fs::mount_part;