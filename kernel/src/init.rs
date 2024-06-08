#![feature(abi_x86_interrupt)]


use os_in_rust_common::{bios_mem::{ARDSType, AddressRangeDescriptorStructure}, context::BootContext, printkln, ASSERT};

use crate::{interrupt, memory, sys_call, sys_call_api, thread_management, tss};

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

    // 申请一个内核页
    let addr = memory::malloc_kernel_page(1);
    printkln!("malloc addr: 0x{:x}", addr);

    thread_management::thread_init();

    // 加载TSS
    tss::tss_init();

    // 注册系统调用函数
    sys_call_api::init();

}