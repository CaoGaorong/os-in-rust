#![feature(abi_x86_interrupt)]


use os_in_rust_common::{bios_mem::{ARDSType, AddressRangeDescriptorStructure}, context::BootContext, instruction, printkln, ASSERT};

use crate::{console, console_println, device, filesystem, interrupt, memory, sys_call, thread_management, tss};

#[inline(never)]
pub fn init_all(boot_info: &BootContext) {
    // 初始化中断描述符和中断控制器
    interrupt::init();

    // 得到memory_map
    let memory_map:&mut [AddressRangeDescriptorStructure]  = unsafe {
        core::slice::from_raw_parts_mut(
            boot_info.memory_map_addr as *mut _,
            boot_info.memory_map_len.try_into().unwrap(),
        )
    };
    // 确保获取了memoryMap
    ASSERT!(memory_map.len() != 0);

    // 从其中找到最大的内存块。
    let os_memory_size = memory_map.iter()
    // 筛选出type
    .filter(|m| m.region_type == ARDSType::Usable as u32)
    // addr + len
    .map(|map| map.base_addr + map.len)
    .map(|size| size as u32)
    // 找出最大的
    .max()
    .unwrap();
    
    memory::mem_pool_init(os_memory_size);

    thread_management::thread_init();

    // 加载TSS
    tss::tss_init();

    // 注册系统调用函数
    sys_call::sys_call_api::init();

    // 初始化硬盘ATA通道
    device::ata_init();

    // 给每个分区，安装文件系统
    device::install_filesystem_for_all_part();

    // 初始化文件系统
    filesystem::mount_part("sdb5");

    // 初始化根目录
    filesystem::init_root_dir();

    // 创建文件
    let res = filesystem::create_file_in_root("test.txt");
    printkln!("create test.txt: {:?}", res);

    let res = filesystem::create_file_in_root("a.txt");
    printkln!("create a.txt: {:?}", res);

    let res = filesystem::create_file_in_root("b.txt");
    printkln!("create b.txt: {:?}", res);


    let result = filesystem::dir::search("/a.txt");
    ASSERT!(result.is_ok());
    let result = result.unwrap();
    printkln!("file name: {}", result.get_name());

    let res = filesystem::dir::mkdir_in_root("sample");
    printkln!("res:{:?}", res);

    let res = filesystem::dir::mkdir_p_in_root("/dev/proc/");
    printkln!("mkdir result: {:?}", res);

    let result = filesystem::dir::search("/dev/proc");
    printkln!("search result:{:?}", result)

    
}