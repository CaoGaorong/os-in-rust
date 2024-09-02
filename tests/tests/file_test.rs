mod test {
    use tests::file_system;

    #[test]
    pub fn test_filepath_split() {
        let path = "/folder/subfolder/file1";
        let mut split = path.split("/");
        loop {
            let item = split.next();
            if item.is_none() {
                break;
            }
            let item = item.unwrap();
            println!("{}", item);
        }
    }

    #[test]
    pub fn search_file_test() {
        let search_res = file_system::search_file("/dev/proc/test.rs");
        println!("found: {}, searched path: {:?}", search_res.0, search_res.1);
    }

    #[test]
    pub fn read_file_test() {
        // 读取/a.txt文件
        let file_data = file_system::read_file("/test");
        if file_data.is_none() {
            println!("no data found");
            return;
        }
        // 转成字符串，打印
        let file_data = file_data.unwrap();
        let string = String::from_utf8(file_data);
        println!("{:?}", string);

    }
}