use core::mem::size_of;

use os_in_rust_common::{
    constants, elem2entry, linked_list::LinkedList, memory, println, racy_cell::RacyCell, thread::{self, PcbPage, TaskStatus, TaskStruct, ThreadArg, ThreadFunc}, utils, ASSERT
};

/**
 * 所有的线程list
 */
static ALL_THREAD_LIST: RacyCell<LinkedList> = RacyCell::new(LinkedList::new());

/**
 * 就绪的进程List
 */
static READY_THREAD_LIST: RacyCell<LinkedList> = RacyCell::new(LinkedList::new());

/**
 * 记得一定要初始化。为什么在new的时候不初始化呢？
 * 因为new设置为const函数，没法获取可变引用
 * 并且，会出现悬空指针
 */
pub fn init_list() {
    unsafe { ALL_THREAD_LIST.get_mut().init() };
    unsafe { READY_THREAD_LIST.get_mut().init() };
}

/**
 * 把当前正在执行的执行流，封装成main线程
 */
pub fn make_thread_main() {
    let all_thread_list = unsafe { ALL_THREAD_LIST.get_mut() };
    // 根据当前运行的线程，找到PCB
    let pcb_page = thread::current_thread();
    // 初始化PCB数据
    pcb_page.init_task_struct(constants::MAIN_THREAD_NAME, 31);

    // main线程，设置为运行中
    pcb_page.task_struct.task_status = TaskStatus::TaskRunning;

    // 添加到所有的进程中
    all_thread_list.append(&mut pcb_page.task_struct.all_tag);
}
pub fn print_thread() {
    println!("all thread:");
    unsafe {
        ALL_THREAD_LIST.get_mut().iter().for_each(|node| {
            let task_struct = elem2entry!(TaskStruct, all_tag, node);
            println!("thread name: {}", unsafe { (&*task_struct).name });
        })
    };
    println!("ready thread:");
    unsafe {
        READY_THREAD_LIST.get_mut().iter().for_each(|node| {
            let task_struct = elem2entry!(TaskStruct, ready_tag, node);
            println!("thread name: {}", unsafe { (&*task_struct).name });
        })
    };
}

/**
 * 启动一个内核线程
 */
pub fn thread_start(
    thread_name: &'static str,
    priority: u8,
    func: ThreadFunc,
    arg: ThreadArg,
) -> *mut PcbPage {
    // 先确保，目前的PCB页大小构建正确。如果错误，请及时修复代码
    ASSERT!(size_of::<PcbPage>() as u32 == constants::PAGE_SIZE);

    // 申请1页内核内存
    let page_addr = memory::malloc_kernel_page(1);

    // 把内核内存转成PCB页
    let pcb_page: &mut PcbPage = unsafe { &mut *(page_addr as *mut PcbPage) };

    // 构建PCB页
    pcb_page.init_task_struct(thread_name, priority);

    // 填充PCB页的中断栈
    pcb_page.init_thread_stack(func, arg);

    // 加入全部线程list
    unsafe {
        ALL_THREAD_LIST
            .get_mut()
            .append(&mut pcb_page.task_struct.all_tag)
    };

    // 添加到就绪线程
    unsafe {
        READY_THREAD_LIST
            .get_mut()
            .append(&mut pcb_page.task_struct.ready_tag)
    };

    pcb_page
}
