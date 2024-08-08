pub mod test {
    use kernel::page_util;

    #[test]
    fn test_split_util() {
        
    }
    #[test]
    fn test_page_table_util() {
        let addr: usize = 0x12345678;
        println!("pde idx: 0x{:x}", page_util::locate_pde(addr.try_into().unwrap()));
        println!("pde idx: 0x{:x}", addr >> 22);
        println!("pte idx: 0x{:x}", page_util::locate_pte(addr.try_into().unwrap()));
    }
}