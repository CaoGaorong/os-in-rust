use os_in_rust_common::{bios_mem::{ARDSType, AddressRangeDescriptorStructure}, context::BootContext, ASSERT, printkln};

use crate::{device, filesystem::{self, File, FileType, OpenOptions, SeekFrom}, interrupt, memory, sys_call, thread_management, tss};

static text: &'static str = "012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789";

#[inline(never)]
#[no_mangle]
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
    let res = File::create("/test.txt");
    printkln!("create test.txt: {:?}", res);
    // 重复创建，失败
    printkln!("{:?}", File::create("/test.txt"));

    // 创建一个父目录不存在的文件
    printkln!("{:?}", File::create("/dev/proc/test.rs"));

    printkln!("create a.txt: {:?}", File::create("/a.txt"));
    printkln!("create b.txt: {:?}", File::create("/b.txt"));


    printkln!("res:{:?}", filesystem::create_dir("/sample"));

    printkln!("mkdir result: {:?}", filesystem::create_dir_all("/dev/proc/"));


    printkln!("create result: {:?}", File::create("/dev/proc/test.rs"));
    printkln!("create result: {:?}", File::create("/dev/proc/test.rs"));
    printkln!("res:{:?}", filesystem::create_dir("/dev/proc/folder1"));

    let dir = filesystem::read_dir("/dev/proc/");
    let dir = dir.unwrap();
    printkln!("read dir: {:?}", dir.get_path());
    let iterator = dir.iter();
    printkln!("read dir: {:?}", iterator);
    for dir_entry in  iterator {
        printkln!("entry_name: {:?}, file_type: {:?}", dir_entry.get_name(), dir_entry.file_type as FileType);
    }


    // let file = OpenOptions::new().read(true).write(true).open("/a.txt");
    // printkln!("open_result: {:?}", file);
    // ASSERT!(file.is_ok());
    // let mut file = file.unwrap();
    // let write_res = file.write(text.as_bytes());
    // printkln!("write file res: {:?}", write_res);

    // let write_res = file.write(text.as_bytes());
    // printkln!("write file res:  {:?}", write_res);

    // let buff: &mut [u8; 666] = memory::malloc(666);
    // printkln!("seek res: {:?}", file.seek(SeekFrom::Start(55)));

    // let read_result = file.read(buff);
    // printkln!("read result: {:?}", read_result);

    // let string = core::str::from_utf8(buff);
    // printkln!("string result: {:?}", string.unwrap());
    
}