mod test {
    use std::{fs::{File, OpenOptions}, io::{BufReader, Read, Seek, Write}};

    use os_in_rust_common::constants;

    const DISK_FILE_PATH: &str = "/Users/jackson/MyProjects/rust/os-in-rust/build/hd60M.img";


    #[test]
    pub fn write_text_file() {
        let mut disk_file = OpenOptions::new().write(true).open(DISK_FILE_PATH).expect("failed to open disk file");
        disk_file.seek(std::io::SeekFrom::Start(constants::DISK_SECTOR_SIZE as u64 * 410)).expect("failed to seek disk file");

        let mut text_file = File::open("/Users/jackson/MyProjects/rust/os-in-rust/cat/src/main.rs").expect("failed to open disk file");
        let mut text = String::new();
        text_file.read_to_string(&mut text).expect("failed to read file");

        println!("text:{}, len:{}", text, text.len());
        disk_file.write(text.as_bytes()).expect("failed to write file ");
        
    }
}