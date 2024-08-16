use core::arch::asm;

use os_in_rust_common::{constants, instruction, paging::{PageTable, PageTableEntry}};

use crate::{interrupt, println, memory::{self, page_util}, pid_allocator, sys_call::sys_call_proxy, thread::{self, ThreadArg}, thread_management};

/**
 * 用户进程的实现
 */
#[cfg(all(not(test), target_arch = "x86"))]
pub extern "C" fn start_process(func_addr: ThreadArg) {
    let pcb_page = thread::current_thread();

    // console_println!("user process:{}", pcb_page.task_struct.name);
    // 申请一个用户页，作为栈空间
    memory::malloc_user_page_by_vaddr(&mut pcb_page.task_struct.vaddr_pool, constants::USER_STACK_TOP_ADDR);

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
#[inline(never)]
pub fn create_page_dir() -> *mut PageTable {
    // 用户进程的页表，用户进程本身不能访问。所以在内核空间申请
    let new_page_table = unsafe { &mut *(memory::malloc_kernel_page(1) as *mut PageTable) };
    
    // 得到当前正在使用的页表
    let cur_page_table = page_util::addr_to_dir_table();
    // 把内核页目录表的第0x300（768）项开始的0x100（256）项，都拷贝到本页表中
    new_page_table.copy_from(cur_page_table, 0x300, 0x100 - 1);

    // 得到这个页表的物理地址
    let page_dir_phy_addr = page_util::get_phy_from_virtual_addr(new_page_table as *const _ as usize);
    // 页表的最后一项，指向自己
    new_page_table.set_entry(new_page_table.size() - 1, PageTableEntry::new_default(page_dir_phy_addr));
    
    new_page_table
}


#[inline(never)]
pub fn process_execute(process_name: &'static str, func: extern "C" fn()) {
    // 申请1页空间
    let pcb_page_addr = memory::malloc_kernel_page(1);
    // 强转
    let pcb_page = unsafe { &mut *(pcb_page_addr as *mut thread::PcbPage) };
    // 初始化任务信息
    pcb_page.init_task_struct(pid_allocator::allocate(), process_name, constants::TASK_DEFAULT_PRIORITY, pcb_page_addr as u32);
    
    // 设置用户地址池
    pcb_page.task_struct.vaddr_pool = thread_management::apply_user_addr_pool();

    // 设置线程栈
    pcb_page.init_thread_stack(start_process, func as u32);

    // 申请1页空间作为该进程的页表
    pcb_page.task_struct.pgdir = create_page_dir();

    // 用户进程有单独的内存块分配器
    pcb_page.task_struct.mem_block_allocator = memory::MemBlockAllocator::new();


    let old_status = instruction::disable_interrupt();

    // 加入全部任务队列
    thread::append_all_thread(&mut pcb_page.task_struct);
    
    // 加入就绪任务队列
    thread::append_read_thread(&mut pcb_page.task_struct);

    // println!("pcb_page:{}", pcb_page);
    instruction::set_interrupt(old_status);

}

/**
 * init程序。pid为1，首个任务
 */
extern "C" fn init_process() {

    // 发起系统调用，fork
    let fork_res = sys_call_proxy::fork();
    match fork_res {
        sys_call_proxy::ForkResult::Parent(child_pid) => {
            println!("i'm father, my pid is {}, my child pid is {}", sys_call_proxy::get_pid().get_data(), child_pid.get_data());
        },
        sys_call_proxy::ForkResult::Child => {
            println!("im child, my pid is {}", sys_call_proxy::get_pid().get_data());
        },
    }
    println!("finish fork");

    // 把时间片让出去，不要空转
    loop {
        sys_call_proxy::thread_yield();
    }
}

/**
 * 做一个假的sleep
 */
fn dummy_sleep(instruction_cnt: u32) {
    for _ in 0 .. instruction_cnt {
        unsafe {asm!("nop");}
    }
}

#[inline(never)]
pub fn init() {
    instruction::disable_interrupt();
    // 执行init进程
    self::process_execute("init", init_process);
}