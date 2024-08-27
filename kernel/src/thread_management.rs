use core::mem::size_of;

use os_in_rust_common::{
    bitmap::BitMap, constants, instruction, pool::MemPool, utils, ASSERT
};

use crate::{memory, pid_allocator, scheduler, thread::{self, PcbPage, TaskStatus, TaskStruct, ThreadArg, ThreadFunc}};


#[inline(never)]
pub fn thread_init() {
    // 主线程
    self::make_thread_main();

    // 设置idle线程
    self::make_idle_thread();
}

/**
 * 把当前正在执行的执行流，封装成main线程
 */
#[inline(never)]
fn make_thread_main() {
    // 根据当前运行的线程，找到PCB
    let pcb_page = thread::current_thread();
    // 初始化PCB数据
    pcb_page.init_task_struct(pid_allocator::allocate(), constants::MAIN_THREAD_NAME, constants::TASK_DEFAULT_PRIORITY, pcb_page as *const _ as u32);

    // main线程，设置为运行中
    pcb_page.task_struct.task_status = TaskStatus::TaskRunning;

    // 添加到所有的进程中
    let main_task = &mut pcb_page.task_struct;
    thread::append_all_thread(main_task);
}

#[inline(never)]
fn make_idle_thread() {
    // 创建idle线程
    let idle_thread = thread_start(constants::IDLE_THREAD_NAME, constants::TASK_DEFAULT_PRIORITY, idle_thread, 0);
    let idle_task = &mut idle_thread.task_struct;
    idle_task.set_status(TaskStatus::TaskWaiting);
    // 填充
    thread::set_idle_thread(idle_task);
}

/**
 * idle线程，只会执行hlt指令
 */
#[inline(never)]
extern "C" fn idle_thread(unused: ThreadArg) {
    loop {
        // 阻塞当前线程
        let cur_task = &mut thread::current_thread().task_struct;
        scheduler::block_thread(cur_task, TaskStatus::TaskBlocked);
        // hlt
        instruction::halt();
    }
}



// pub fn print_thread() {
//     printkln!("all thread:");
//     unsafe {
//         ALL_THREAD_LIST.get_mut().iter().for_each(|node| {
//             let task_struct = elem2entry!(TaskStruct, all_tag, node);
//             printkln!("thread name: {}", unsafe { (&*task_struct).name });
//             printkln!("task struct addr:0x{:x}", task_struct as u32);
//             printkln!("task struct stack addr:0x{:x}", (&*task_struct).kernel_stack as u32);
//         })
//     };
    
// }

/**
 * 启动一个内核线程
 */
#[inline(never)]
pub fn thread_start(
    thread_name: &'static str,
    priority: u8,
    func: ThreadFunc,
    arg: ThreadArg,
) -> &'static mut PcbPage {
    // 先确保，目前的PCB页大小构建正确。如果错误，请及时修复代码
    ASSERT!(size_of::<PcbPage>() as u32 == constants::PAGE_SIZE);

    // 申请1页内核内存
    let page_addr = memory::malloc_kernel_page(1);

    // 把内核内存转成PCB页
    let pcb_page: &'static mut PcbPage = unsafe { &mut *(page_addr as *mut PcbPage) };

    // 构建PCB页
    pcb_page.init_task_struct(pid_allocator::allocate(), thread_name, priority, page_addr as u32);

    // 填充PCB页的中断栈
    pcb_page.init_thread_stack(func, arg);

    let task = &mut pcb_page.task_struct;
    // 加入全部线程list
    thread::append_all_thread(task);

    // 添加到就绪线程
    thread::append_read_thread(task);

    pcb_page
}


/**
 * 让当前任务让出CPU，重新进入就绪队列
 */
pub fn thread_yield() {
    let pcb_page = thread::current_thread();
    let cur_task = &mut pcb_page.task_struct;

    // 关闭中断。防止该线程重复加入就绪队列
    let old_status = instruction::disable_interrupt();
    ASSERT!(!thread::get_ready_thread().contains(&cur_task.general_tag));
    
    // 当前线程加入就绪队列
    thread::append_read_thread(cur_task);
    cur_task.set_status(TaskStatus::TaskReady);

    // 切换到其他线程执行
    scheduler::schedule();

    // 恢复中断
    instruction::set_interrupt(old_status);
}

/**
 * 申请用户进程虚拟地址池
 * 关键在于向堆空间申请，作为位图
 */
#[inline(never)]
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
    let bitmap_array = unsafe { core::slice::from_raw_parts_mut(bitmap_addr as *mut u8, bitmap_byte_len) };
    // 进程的虚拟地址池
    MemPool::new(constants::USER_PROCESS_ADDR_START, BitMap::new(bitmap_array))
}



/**
 * 某个线程退出
 */
#[inline(never)]
pub fn free_thread(task: &mut TaskStruct) {
    let old_status = instruction::disable_interrupt();
    task.task_status = TaskStatus::TaskDied;

    // 从就绪队列移除
    thread::remove_from_ready_thread(task);
    // 从全部队列移除
    thread::remove_from_all_thread(task);

    // 释放pid
    pid_allocator::release(task.pid);

    // 把这PCB占用的一整页内核空间给释放掉
    memory::free_kernel_page(task as *const _ as usize, 1, true);

    instruction::set_interrupt(old_status);
}