use os_in_rust_common::{bios_mem::{ARDSType, AddressRangeDescriptorStructure}, context::BootContext, instruction, printkln, ASSERT};

use crate::{device, filesystem, interrupt, memory, process, sys_call, thread, thread_management, tss};

#[inline(never)]
pub fn init_all(boot_info: &BootContext) {
    // 初始化中断描述符和中断控制器
    interrupt::init();

    // 得到memory_map
    let memory_map:&mut [AddressRangeDescriptorStructure]  = unsafe {
        core::slice::from_raw_parts_mut(
            boot_info.memory_map_addr as *mut AddressRangeDescriptorStructure,
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

    // init进程初始化
    process::init();

    // 线程初始化（main和idle初始化）
    thread_management::thread_init();
    thread::check_task_stack("failed to init thread");

    // 加载TSS
    tss::tss_init();

    thread::check_task_stack("overflow after tss init");

    // 注册系统调用函数
    sys_call::init();
    thread::check_task_stack("overflow after syscall init");

    // 初始化硬盘ATA通道
    device::ata_init();
    thread::check_task_stack("overflow after ata init");

    // 给每个分区，安装文件系统
    filesystem::install_filesystem_for_all_part();
    thread::check_task_stack("overflow after fs init");

    // 初始化文件系统
    filesystem::mount_part("sdb5");
    thread::check_task_stack("overflow after fs mounted");

    // 初始化根目录
    filesystem::init_root_dir();
    thread::check_task_stack("overflow after root dir init");
}