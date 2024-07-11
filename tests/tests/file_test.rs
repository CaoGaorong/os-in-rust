mod test {
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
}