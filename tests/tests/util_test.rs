pub mod test {
    use std::hint::spin_loop;

    use kernel::memory::page_util;
    use os_in_rust_common::{constants, cstring_utils};

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

    #[test]
    pub fn read_task_name() {
        println!("{:?}", cstring_utils::read_from_bytes(&[109, 97, 105, 110, 0]));
        println!("{:?}", cstring_utils::read_from_bytes(&[105, 110, 105, 116, 0]));
        println!("{:?}", cstring_utils::read_from_bytes(&[105, 100, 108, 101, 0]));
    }

    #[test]
    pub fn test_split() {
        let split = "A | B| C |D".split("|");
        for ele in split {
            println!("{}", ele);
        }
    }
    #[test]
    pub fn test() {
        let s = format_args!("x{}", 123).as_str();
        println!("s:{}", s.unwrap());

    }
}