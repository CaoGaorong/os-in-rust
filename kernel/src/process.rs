use core::{mem, ptr::slice_from_raw_parts, slice};

use os_in_rust_common::{bitmap::BitMap, constants, instruction, paging::{self, PageTable, PageTableEntry}, pool::MemPool};

use crate::{memory, page_util, thread::{self, TaskStruct}, thread_management};

/**
 * 用户进程的实现
 */
pub fn start_process(func: ThreadArg) {
    let cur_task = &mut thread::current_thread().task_struct;
    cur_task.kernel_stack += size_of::<thread::ThreadStack>();
    let intr_stack = cur_task.kernel_stack as *mut thread::InterruptStack as &mut thread::InterruptStack;
    // intr_stack.
    
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
    page_table.copy_from(paging::get_dir_ref(), 0x300, 0x100 * 4);
    
    // 得到这个页表的物理地址
    let page_dir_phy_addr = page_util::get_phy_from_virtual_addr(page_dir_addr);
    // 页表的最后一项，指向自己
    page_table.set_entry(page_table.size() - 1, PageTableEntry::new_default(page_dir_phy_addr))
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
    let bitmap_bit_len = (virtual_addr_len / constants::PAGE_SIZE) as usize;
    // 位图中一共需要bitmap_byte_len个字节
    let bitmap_byte_len = (bitmap_bit_len / 8) as usize;
    // 该位图一共需要bitmap_page_cnt页
    let bitmap_page_cnt =  (bitmap_byte_len / constants::PAGE_SIZE) as usize;
    
    /**** 2. 申请堆空间 */
    // 向堆空间申请空间
    let bitmap_addr = memory::malloc_kernel_page(bitmap_page_cnt);
    
    /**** 3. 把申请到的堆空间，构建一个虚拟地址池 */
    // 把这一块空间，转成一个数组的引用
    let bitmap_array = unsafe { &*slice_from_raw_parts(bitmap_addr as *const u8, bitmap_byte_len) };
    // 进程的虚拟地址池
    MemPool::new(constants::USER_PROCESS_ADDR_START, BitMap::new(bitmap_array))
}

pub fn process_execute(process_name: &str) {
    // 申请1页空间
    let pcb_page_addr = memory::malloc_kernel_page(1);
    // 强转
    let pcb_page = unsafe { &mut *(pcb_page_addr as *mut thread::PcbPage) };
    // 初始化任务信息
    pcb_page.init_task_struct(process_name, constants::TASK_DEFAULT_PRIORITY);
    
    pcb_page.init_thread_stack(function, arg)

    // 申请1页空间作为该进程的页表
    pcb_page.task_struct.pgdir = create_page_dir();

    // 设置用户地址池
    pcb_page.task_struct.vaddr_pool = apply_user_addr_pool();

    let old_status = instruction::disable_interrupt();

    // 加入全部任务队列
    thread_management::get_all_thread().append(&mut pcb_page.task_struct.all_tag);
    
    // 加入就绪任务队列
    thread_management::get_ready_thread().append(&mut pcb_page.task_struct.general_tag);

    instruction::set_interrupt(old_status);

}