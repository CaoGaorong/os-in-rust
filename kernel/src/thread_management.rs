use core::{mem::size_of, ptr, task};

use os_in_rust_common::{
    constants, elem2entry, instruction, linked_list::LinkedList, printkln, racy_cell::RacyCell, ASSERT, MY_PANIC
};

use crate::{memory, scheduler, sync::Lock, thread::{self, PcbPage, TaskStatus, TaskStruct, ThreadArg, ThreadFunc}};


/**
 * 记得一定要初始化。为什么在new的时候不初始化呢？
 * 因为new设置为const函数，没法获取可变引用
 * 并且，会出现悬空指针
 */
#[inline(never)]
pub fn thread_init() {
    // unsafe { ALL_THREAD_LIST.get_mut().init() };
    // unsafe { READY_THREAD_LIST.get_mut().init() };
    // 主线程
    make_thread_main();

    // 设置idle线程
    make_idle_thread();
}

/**
 * 把当前正在执行的执行流，封装成main线程
 */
pub fn make_thread_main() {
    // 根据当前运行的线程，找到PCB
    let pcb_page = thread::current_thread();
    // 初始化PCB数据
    pcb_page.init_task_struct(constants::MAIN_THREAD_NAME, constants::TASK_DEFAULT_PRIORITY, pcb_page as *const _ as u32);

    // main线程，设置为运行中
    pcb_page.task_struct.task_status = TaskStatus::TaskRunning;

    // 添加到所有的进程中
    let main_task = &mut pcb_page.task_struct;
    thread::append_all_thread(main_task);
}

pub fn make_idle_thread() {
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
extern "C" fn idle_thread(unused: ThreadArg) {
    loop {
        // 阻塞当前线程
        let cur_task = &mut thread::current_thread().task_struct;
        block_thread(cur_task, TaskStatus::TaskBlocked);
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
    pcb_page.init_task_struct(thread_name, priority, page_addr as u32);

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
 * 阻塞某一个线程。（一般阻塞操作都是线程阻塞自身）
 *      这里利用关闭中断和开启中断，来实现方法的原子性。先disable_interrupt，然后恢复中断
 *      - let old_status = disable_interrupt();
 *      - set_interrupt(old_status);
 * 
 * 关于这个操作本身有没有并发问题？
 *      但是其实没有并发问题，因为当disable_interrupt()的cti指令执行后，就不存在线程切换，就不可能有其他线程来抢夺
 *      因此如果是cti指令前并发（当前线程被切走），那没关系，当该线程被切回来后，会接着执行cti操作，不影响后续
 *      如果是cti指令执行之后，那更不可能并发，因为不会切换线程了
 */
pub fn block_thread(task: &mut TaskStruct, task_status: TaskStatus) {
    // 只能是这三种状态之一
    let allow_status = [TaskStatus::TaskBlocked, TaskStatus::TaskHanging, TaskStatus::TaskWaiting];
    ASSERT!(allow_status.contains(&task_status));
    
    // 关闭中断
    let old_status = instruction::disable_interrupt();
    // 设置任务位阻塞状态
    task.set_status(task_status);
    // 切换线程
    scheduler::schedule();
    // 恢复中断 - 被唤醒之后的操作
    instruction::set_interrupt(old_status);
}

// /**
//  * 唤醒某一个线程。
//  * 某个阻塞的线程只能被其他线程唤醒
//  */
// #[inline(never)]
// pub fn wake_thread(task: &mut TaskStruct)  {
//     // 关闭中断
//     let old_status = instruction::disable_interrupt();
//     // 只能是这三种状态之一
//     let allow_status = [TaskStatus::TaskBlocked, TaskStatus::TaskHanging, TaskStatus::TaskWaiting];
//     if !allow_status.contains(&task.task_status) {
//         MY_PANIC!("could not wake up thread; name:{}, status:{:?}", task.name, &task.task_status);
//     }

//     ASSERT!(task.task_status != TaskStatus::TaskReady);
//     // 设置为就绪状态
//     task.task_status = TaskStatus::TaskReady;
//     // 放入就绪队列
//     thread::append_read_thread(task);

//     // 恢复中断
//     instruction::set_interrupt(old_status);
// }

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