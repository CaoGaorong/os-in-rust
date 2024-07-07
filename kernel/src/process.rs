use core::{arch::{asm, global_asm}, mem::{self, size_of}, ptr::slice_from_raw_parts, slice};

use os_in_rust_common::{bitmap::BitMap, constants, instruction, paging::{self, PageTable, PageTableEntry}, pool::MemPool, printkln, utils};

use crate::{console_println, interrupt, memory, mutex::Mutex, page_util, thread::{self, TaskStruct, ThreadArg}, thread_management};

/**
 * 用户进程的实现
 */
#[cfg(all(not(test), target_arch = "x86"))]
pub extern "C" fn start_process(func_addr: ThreadArg) {
    let pcb_page = thread::current_thread();

    // console_println!("user process:{}", pcb_page.task_struct.name);
    // 申请一个用户页，作为栈空间
    memory::malloc_user_stack_page(constants::USER_STACK_TOP_ADDR);

    pcb_page.init_intr_stack(func_addr, constants::USER_STACK_BASE_ADDR as u32);

    let pcb_intr_stack_addr = &(pcb_page.interrupt_stack) as *const _ as u32;
    pcb_page.task_struct.kernel_stack = pcb_intr_stack_addr;
    

    // 把栈顶，指向中断栈的低地址处，准备恢复中断栈的上下文
    unsafe {
        asm!(
            "mov esp, {:e}",
            in(reg) pcb_intr_stack_addr
        )
    }
    // 退出中断，恢复上下文数据
    interrupt::intr_exit();
}

#[cfg(all(not(target_arch = "x86")))]
pub extern "C" fn start_process(func_addr: ThreadArg) {
    todo!()
}


/**
 * 创建一个页目录表
 */
pub fn create_page_dir() -> *mut PageTable {
    // 用户进程的页表，用户进程本身不能访问。所以在内核空间申请
    let page_dir_addr = memory::malloc_kernel_page(1);
    
    // 把这个地址空间，转成一个页表的格式。空的页表
    let mut page_table = paging::PageTable::from(page_dir_addr);
    // 把内核页目录表的第0x300（768）项开始的0x100（256）项，都拷贝到本页表中

    // 得到页表的虚拟地址。「但是我们需要访问页表自身，而不是通过页表访问映射到的内存」
    let kernel_page_dir_addr = constants::KERNEL_PAGE_DIR_ADDR;
    // 所以根据这个地址，构造到一个可以访问到「页表自身」的虚拟地址
    let kernel_page_dir = unsafe { &*page_util::addr_to_dir_table() };
    // 用这个虚拟地址来访问「内核页表自身」
    page_table.copy_from(kernel_page_dir, 0x300, 0x100 - 1);

    // 得到这个页表的物理地址
    let page_dir_phy_addr = page_util::get_phy_from_virtual_addr(page_dir_addr);
    // 页表的最后一项，指向自己
    page_table.set_entry(page_table.size() - 1, PageTableEntry::new_default(page_dir_phy_addr));
    
    page_table
}

/**
 * 申请用户进程虚拟地址池
 * 关键在于向堆空间申请，作为位图
 */
pub fn apply_user_addr_pool() -> MemPool {
    /**** 1. 计算位图需要多大的堆空间 */
    // 虚拟地址的长度。单位字节
    let virtual_addr_len = constants::KERNEL_ADDR_START - constants::USER_PROCESS_ADDR_START;
    // 位图的1位，代表一页虚拟地址。那么位图中一共需要bitmap_bit_len位
    let bitmap_bit_len = utils::div_ceil(virtual_addr_len as u32, constants::PAGE_SIZE as u32) as usize;
    // 位图中一共需要bitmap_byte_len个字节
    let bitmap_byte_len = utils::div_ceil(bitmap_bit_len as u32, 8) as usize;
    // 该位图一共需要bitmap_page_cnt页
    let bitmap_page_cnt =  utils::div_ceil(bitmap_byte_len as u32, constants::PAGE_SIZE as u32) as usize; 
    
    /**** 2. 申请堆空间 */
    // 向堆空间申请空间
    let bitmap_addr = memory::malloc_kernel_page(bitmap_page_cnt);
    
    /**** 3. 把申请到的堆空间，构建一个虚拟地址池 */
    // 把这一块空间，转成一个数组的引用
    let bitmap_array = unsafe { &*slice_from_raw_parts(bitmap_addr as *const u8, bitmap_byte_len) };
    // 进程的虚拟地址池
    MemPool::new(constants::USER_PROCESS_ADDR_START, BitMap::new(bitmap_array))
}

pub fn process_execute(process_name: &'static str, func: extern "C" fn()) {
    // 申请1页空间
    let pcb_page_addr = memory::malloc_kernel_page(1);
    // 强转
    let pcb_page = unsafe { &mut *(pcb_page_addr as *mut thread::PcbPage) };
    // 初始化任务信息
    pcb_page.init_task_struct(process_name, constants::TASK_DEFAULT_PRIORITY, pcb_page_addr as u32);
    
    // 设置用户地址池
    pcb_page.task_struct.vaddr_pool = apply_user_addr_pool();

    // 设置线程栈
    pcb_page.init_thread_stack(start_process, func as u32);

    // 申请1页空间作为该进程的页表
    pcb_page.task_struct.pgdir = create_page_dir();

    // 用户进程有单独的内存块分配器
    pcb_page.task_struct.mem_block_allocator = memory::mem_block::MemBlockAllocator::new();


    let old_status = instruction::disable_interrupt();

    // 加入全部任务队列
    thread_management::get_all_thread().append(&mut pcb_page.task_struct.all_tag);
    
    // 加入就绪任务队列
    thread_management::append_read_thread(&mut pcb_page.task_struct);

    // println!("pcb_page:{}", pcb_page);
    instruction::set_interrupt(old_status);

}