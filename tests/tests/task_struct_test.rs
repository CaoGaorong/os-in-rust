mod test {
    use std::mem::size_of;

    use kernel::thread::TaskStruct;

    #[test]
    pub fn task_struct_size() {
        println!("size: 0x{:x}", size_of::<TaskStruct>());
        println!("max addr in task: 0x{:x}", 0xc009e000 + size_of::<TaskStruct>());
    }
}