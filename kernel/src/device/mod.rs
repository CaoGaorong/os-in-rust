mod constant;
pub mod ata;
pub mod init;
pub mod pio;
pub mod drive;

pub use init::ata_init;
pub use init::get_ata_channel;