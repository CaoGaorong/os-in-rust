#[cfg(test)]
mod tests {
    use os_in_rust_common::paging;
    #[test]
    fn test_set_entry() {
        let entry = paging::PageTableEntry::new_default(0x1234);
        println!("entry: {}", unsafe {*(&entry as *const paging::PageTableEntry as *const u32)});
    }
}