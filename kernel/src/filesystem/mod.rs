pub mod constant;
pub mod superblock;
pub mod inode;
pub mod dir;
pub mod init;
pub mod file;
pub mod file_descriptor;
pub mod global_file_table;
pub mod fs;

pub use init::init;
pub use init::install_filesystem;
pub use init::mount_part;
pub use dir::init_root_dir;
pub use init::create_file_in_root;