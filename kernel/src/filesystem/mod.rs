mod constant;
pub mod superblock;
pub mod inode;
pub mod dir;
pub mod fs;

pub use fs::init;
pub use fs::install_filesystem;
pub use fs::mount_part;