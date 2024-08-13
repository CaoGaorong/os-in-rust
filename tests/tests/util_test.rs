pub mod test {
    use kernel::memory::page_util;
    use os_in_rust_common::constants;

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

    #[test]
    pub fn bit_test() {
        println!("{:b}", 0b11 & !0x0)
    }

    #[test]
    pub fn user_stack_test() {
        let end_idx = page_util::locate_pde(constants::USER_STACK_BASE_ADDR);
        let start_idx = page_util::locate_pde(constants::USER_PROCESS_ADDR_START);
        println!("[{}, {})", start_idx, end_idx);

        println!("{}", page_util::locate_pte(constants::USER_PROCESS_ADDR_START))
    }
}