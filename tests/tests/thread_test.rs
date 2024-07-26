mod test {
    use std::mem::size_of;

    use kernel::thread::TaskStruct;

    #[test]
    pub fn test_task_size() {
        let task_size = size_of::<TaskStruct>();
        println!("task_size: {}", task_size);
    }
}