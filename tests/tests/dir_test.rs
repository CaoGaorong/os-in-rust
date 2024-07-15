mod test {
    #[test]
    fn test_split() {
        let mut entry_list = "/dev/proc/id".split("/");
        while let Some(entry) = entry_list.next() {
            if entry.is_empty() {
                continue;
            }
            println!("entry: {:?}", entry);
        }
    }
}
