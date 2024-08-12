use core::{arch::asm, fmt::Display, mem::size_of, ptr};

use os_in_rust_common::{constants, cstr_write, cstring_utils, domain::InodeNo, elem2entry, instruction::{self, enable_interrupt}, linked_list::{LinkedList, LinkedNode}, paging::{self, PageTable}, pool::MemPool, printkln, racy_cell::RacyCell, reg_cr3::{self, CR3}, reg_eflags::{self, EFlags, FlagEnum}, selector::SegmentSelector, utils, ASSERT, MY_PANIC};

use crate::{console_println, filesystem::FileDescriptorTable, interrupt, memory::{page_util, MemBlockAllocator}, pid_allocator::Pid, tss};


/**
 * 所有的线程list
 */
static ALL_THREAD_LIST: RacyCell<LinkedList> = RacyCell::new(LinkedList::new());

/**
 * 就绪的进程List
 */
static READY_THREAD_LIST: RacyCell<LinkedList> = RacyCell::new(LinkedList::new());

static IDLE_THREAD: RacyCell<Option<&mut TaskStruct>> = RacyCell::new(Option::None);



pub fn get_all_thread() -> &'static mut LinkedList{
    unsafe { ALL_THREAD_LIST.get_mut() }
}

pub fn append_all_thread(thread: &mut TaskStruct) {
    get_all_thread().append(&mut thread.all_tag);
}


pub fn get_ready_thread() -> &'static mut LinkedList{
    unsafe { READY_THREAD_LIST.get_mut() }
}

pub fn append_read_thread(thread: &mut TaskStruct) {
    get_ready_thread().append(&mut thread.general_tag);
}

pub fn set_idle_thread(thread: &'static mut TaskStruct) {
    let idle_thread = unsafe { IDLE_THREAD.get_mut() };
    *idle_thread = Option::Some(thread);
}

pub fn get_idle_thread() -> &'static mut TaskStruct {
    let idle_thread = unsafe { IDLE_THREAD.get_mut() };
    if idle_thread.is_none() {
        MY_PANIC!("idle thread not exist");
    }
    let idle_thread = idle_thread.as_deref_mut();
    idle_thread.unwrap()
}


/**
 * 创建一个内核线程，传递的函数
 */
pub type ThreadFunc = extern "C" fn(ThreadArg);

/**
 * 内核线程，函数执行的参数
 */
pub type ThreadArg = u32;



#[inline(never)]
extern "C" fn kernel_thread_wrapper(function: ThreadFunc, arg: ThreadArg) {
    // 开启中断。线程切换依赖时钟中断
    enable_interrupt();
    function(arg)
}


#[inline(never)]
extern "C" fn kernel_exit_wrapper(function: ThreadFunc, arg: ThreadArg) {
    interrupt::intr_exit();
}




/**
 * 获取当前运行的线程
 */
#[inline(never)]
pub fn current_thread() -> &'static mut PcbPage {
    let cur_esp = instruction::load_esp();
    unsafe { &mut *((cur_esp & 0xfffff000) as *mut PcbPage) }
}

#[inline(never)]
pub fn check_task_stack(msg: &str) {
    let cur_task = &current_thread().task_struct;
    cur_task.check_stack_magic(msg);
}

/**
 * PCB空闲区域的大小 = 1页 - 其他用的区域
 */
const PCB_PAGE_BLANK_SIZE: usize = constants::PAGE_SIZE as usize
    - size_of::<TaskStruct>()
    - size_of::<ThreadStack>()
    - size_of::<InterruptStack>();
/**
 * 一个PCB页的结构（保证是一个物理页占用4KB）
 */
#[repr(C)]
pub struct PcbPage {
    /**
     * 开始部分是PCB
     */
    pub task_struct: TaskStruct,
    /**
     * 中间部分填充0
     */
    zero: [u8; PCB_PAGE_BLANK_SIZE],
    /**
     * 然后是线程栈
     */
    pub thread_stack: ThreadStack,
    /**
     * 一页的部分最后是中断栈
     */
    pub interrupt_stack: InterruptStack,
}
impl PcbPage {
    pub fn new(
        task_struct: TaskStruct,
        thread_stack: ThreadStack,
        interrupt_stack: InterruptStack,
    ) -> Self {
        Self {
            task_struct,
            zero: [0; PCB_PAGE_BLANK_SIZE],
            thread_stack,
            interrupt_stack,
        }
    }

    /**
     * 初始化PCB
     */
    pub fn init_task_struct(&mut self, pid: Pid, name: &'static str, priority: u8, pcb_page_addr: u32) {
        // 线程栈的地址
        let thread_stack_ptr = &mut self.thread_stack as *mut ThreadStack as u32;
        // 初始化任务信息
        self.task_struct.init(pid, name, priority, thread_stack_ptr, pcb_page_addr);
    }

    /**
     * 初始化线程栈
     */
    pub fn init_thread_stack(&mut self, function: ThreadFunc, arg: ThreadArg) {
        self.thread_stack.init(function, arg);
    }

    /**
     * 初始化程序退出的线程栈
     */
    #[inline(never)]
    pub fn  init_exit_thread_stack(&mut self) {
        // 设置线程栈的内容。指向程序结束的地方
        self.thread_stack.init_exit_stack();
        // 线程栈的地址
        let thread_stack_addr = &mut self.thread_stack as *mut ThreadStack as u32;
        // 这里该任务的栈地址，就是线程栈的起始地址
        self.task_struct.kernel_stack = thread_stack_addr;
    }

    pub fn init_intr_stack(&mut self, fun_addr: u32, user_stack_addr: u32) {
        self.interrupt_stack.init(fun_addr, user_stack_addr);
    }

    /**
     * 加载PCB页。
     * 把把“上下文”恢复到当前CPU寄存器中，利用ret指令，把(esp + 4)的值赋值到eip寄存器，来实现跳转执行
     */
    #[cfg(all(not(test), target_arch = "x86"))]
    pub fn do_load(&self) {
        // 当前PCB的栈指针指向的地址
        let stack_addr = self.task_struct.kernel_stack;

        // 把这个栈指针，恢复到esp寄存器，那么就开始执行了
        unsafe {
            asm!(
                "mov esp, {0:e}",
                "pop ebp",
                "pop ebx",
                "pop edi",
                "pop esi",
                "ret", // ret指令一执行，会取出(esp + 4) 这一项作为要执行的函数地址。而(esp + 8)是要执行函数的参数
                in(reg) stack_addr,
            );
        }
    }
    #[cfg(all(not(target_arch = "x86")))]
    pub fn do_load(&self) {
        todo!()
    }
}

impl Display for PcbPage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        console_println!("PcbPage(task_struct: {}, thread_stack:{}, interrupt_stack:{})", self.task_struct, self.thread_stack, self.interrupt_stack);
        Result::Ok(())
    }
}
/**
 * PCB的结构
*/
#[repr(C)]
pub struct TaskStruct {
    /**
     * pid
     */
    pub pid: Pid,
    /**
     * 父任务的pid
     */
    pub parent_pid: Pid,
    /**
     * PCB内核栈地址
     */
    pub kernel_stack: u32,
    /**
     * PCB的名称
     */
    name: [u8; constants::TASK_NAME_LEN],
    /**
     * PCB状态
     */
    pub task_status: TaskStatus,
    /**
     * PCB优先级
     */
    pub priority: u8,

    /**
     * 当前进程/线程还剩的滴答数量
     */
    pub left_ticks: u8,
    /**
     * 该任务一共执行了多久
     */
    pub elapsed_ticks: u8,
    /**
     * 该PCB使用的页表地址
     */
    pub pgdir: *mut PageTable,

    /**
     * 该任务的文件描述符数组
     */
    pub fd_table: FileDescriptorTable,

    /**
     * 该pcb的通用链表tag。
     * 可以放就绪队列，也可以放阻塞队列，所以是通用的
     */
    pub general_tag: LinkedNode,
    /**
     * 全部进程的链表tag
     */
    pub all_tag: LinkedNode,

    /**
     * 该进程的虚拟地址池
     */
    pub vaddr_pool: MemPool,

    /**
     * PCB页的地址
     */
    pub pcb_page_addr: u32,

    /**
     * 内存块分配器。支持分配多种规格的内存块
     */
    pub mem_block_allocator: MemBlockAllocator,

    /**
     * 该进程的工作目录的inode
     */
    pub cwd_inode: Option<InodeNo>,

    /**
     * 栈边界的魔数
     */
    pub stack_magic: u32,
}

// 自己保证并发问题
unsafe impl Send for TaskStruct {}
unsafe impl Sync for TaskStruct {}

impl Display for TaskStruct {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        console_println!("TaskStruct(name:{}, kernel_stack:0x{:x}, task_status:{:?}, pgdir:0x{:x}, vaddr_pool:{})", self.get_name(), self.kernel_stack as u32, self.task_status, self.pgdir as u32, self.vaddr_pool);;
        Result::Ok(())
    }
}

impl TaskStruct {
    #[inline(never)]
    fn init(&mut self, pid: Pid, name: &str, priority: u8, kernel_stack: u32, pcb_page_addr: u32) {
        self.pid = pid;
        self.kernel_stack = kernel_stack;
        cstr_write!(&mut self.name, "{}", name);
        self.task_status = TaskStatus::TaskReady;
        self.priority = priority;
        self.stack_magic = constants::TASK_STRUCT_STACK_MAGIC;
        self.left_ticks = priority;
        self.elapsed_ticks = 0;
        self.pgdir = ptr::null_mut();
        self.general_tag = LinkedNode::new();
        self.all_tag = LinkedNode::new();
        self.pcb_page_addr = pcb_page_addr;
        self.fd_table = FileDescriptorTable::new();
    }

    #[inline(never)]
    pub fn get_name(&self) -> &str {
        self.check_stack_magic("error to get task name");
        let task_name = cstring_utils::read_from_bytes(&self.name);
        ASSERT!(task_name.is_some());
        task_name.unwrap()
    }

    pub fn set_status(&mut self , status: TaskStatus) {
        self.task_status = status;
    }

    /**
     * 重置该任务剩余的ticks
     */
    pub fn reset_ticks(&mut self) {
        self.left_ticks = self.priority;
    }

    pub fn activate_process(&mut self) {
        // if self as *const _ as usize == self.pgdir as usize {
        //     self.pgdir = ptr::null_mut();
        // }
        // 激活页表
        self.activate_pgdir();
        if self.pgdir == ptr::null_mut() {
            return;
        }
        // 当前PCB页的最高地址 = PCB起始地址 + PCB页大小
        let pcb_high_addr = self.pcb_page_addr + size_of::<PcbPage>() as u32;
        tss::update_esp0(pcb_high_addr);
    }
    
    /**
     * 激活该任务的页表
     */
    #[no_mangle]
    #[inline(never)]
    fn activate_pgdir(&self) {
        // 如果该任务没有页表
        let page_dir_phy_addr = if self.pgdir.is_null() {
            // 取内核的页表物理地址。
            constants::KERNEL_PAGE_DIR_ADDR
        } else {
            page_util::get_phy_from_virtual_addr(self.pgdir as usize)
        };

        // 把物理地址放入cr3寄存器
        let cr3 = reg_cr3::CR3::new(page_dir_phy_addr as *const PageTable);
        // 加载页表
        cr3.load_cr3();
    }
    
    
    /**
     * 根据all_tag的地址，解析出TaskStruct本身
     */
    pub fn parse_by_all_tag(all_tag: &LinkedNode) -> *mut Self {
        elem2entry!(Self, all_tag, all_tag as *const LinkedNode as usize)
    }

    /** 
     * 根据ready_tag的地址，解析出TaskStruct本身
    */
    pub fn parse_by_general_tag(general_tag: &LinkedNode) -> *mut Self {
        elem2entry!(Self, general_tag, general_tag as *const LinkedNode as usize)
    }

    #[inline(never)]
    pub fn check_stack_magic(&self, msg: &str) {
        // 这里cur_task.pid != 0 是为了防止PCB还未初始化
        if self.pid != Pid::new(0) && self.stack_magic != constants::TASK_STRUCT_STACK_MAGIC {
            MY_PANIC!("thread {} stack overflow, {}", self.get_name(), msg);
        }
    }



}

// #[repr(u32)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TaskStatus {
    TaskRunning,
    TaskReady,
    TaskBlocked,
    TaskWaiting,
    TaskHanging,
    TaskDied,
}

/**
 * 线程栈
 * ebp、ebx、edi、esi是ABI约定需要保存的
 * 
 * **注意，只有任务首次执行，才是这样的结构。
 * ** 因为对于栈，其实没有固定结构的。
 * 
 */
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct ThreadStack {
    /*---C语言ABI的标准———— */
    // ebp: u32,
    // ebx: u32,
    // edi: u32,
    // esi: u32,
    /**
     * 利用ret指令，把这个要执行的函数地址，赋值给eip，从而实现“跳转执行”
     */
    eip: extern "C" fn(ThreadFunc, ThreadArg),
    /* --- 以下三个字段，只有第一次构建要用，利用ret指令“欺骗CPU”------ */
    /**
     * 返回地址。因为ret指令，会默认esp指向的地址处，是调用者的返回地址。但是其实我们不需要，单纯占位用
     */
    ret_addr: *const u8,
    /**
     * 我们真正要执行的函数地址
     */
    function: ThreadFunc,
    /**
     * 真正要执行的函数的参数
     */
    func_arg: ThreadArg,
}
impl ThreadStack {
    /**
     * 构建一个线程栈。
     * function: 要执行的函数
     * arg: 该函数的地址
     */
    #[inline(never)]
    fn new(function: ThreadFunc, arg: ThreadArg) -> Self {
        Self {
            // ebp: 0,
            // ebx: 0,
            // edi: 0,
            // esi: 0,
            eip: kernel_thread_wrapper,
            ret_addr: ptr::null(), // 占位用，没啥用
            function,
            func_arg: arg,
        }
    }
    
    #[inline(never)]
    fn init_exit_stack(&mut self) {
        self.eip = kernel_exit_wrapper;
    }


    #[inline(never)]
    fn init(&mut self, function: ThreadFunc, arg: ThreadArg) {
        // self.ebp = 0;
        // self.ebx = 0;
        // self.edi = 0;
        // self.esi = 0;
        self.eip = kernel_thread_wrapper;
        self.ret_addr = ptr::null();
        self.function = function;
        self.func_arg = arg;
    }
}

impl Display for ThreadStack {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // console_println!("ThreadStack(ebp:0x{:x}, ebx:0x{:x}, edi:0x{:x}, esi:0x{:x}, eip:0x{:x}, ret_addr:0x{:x}, function:0x{:x}, func_arg:0x{:x})", 
        // self.ebp as u32, 
        // self.ebx as u32, 
        // self.edi as u32, 
        // self.esi as u32, 
        // self.eip as u32, 
        // self.ret_addr as u32, 
        // self.function as u32, 
        // self.func_arg as u32);
        Result::Ok(())
    }
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct InterruptStack {
    // 下面几个寄存器，是操作系统实现中，手动赋值的
    edi: u32,
    esi: u32,
    ebp: u32,
    esp_dummy: u32, // pushad自动压入，popad自动弹出
    ebx: u32,
    edx: u32,
    ecx: u32,
    pub eax: u32,
    gs: u16,
    fs: u16,
    es: u16,
    ds: u16,

    // 下面的字段数据，是中断发生CPU自动压入0级栈，中断退出自动弹出栈的
    eip:u32,
    cs: u32,
    eflags: u32,
    esp: u32,
    ss: u32,
}
impl InterruptStack {
    /**
     * fun_addr: 中断返回的函数地址
     * user_stack_addr: 用户栈的虚拟地址
     */
    pub fn new(fun_addr: u32, user_stack_addr: u32) -> Self {
        
        Self {
            edi: 0,
            esi: 0,
            ebp: 0,
            esp_dummy: 0,
            ebx: 0,
            edx: 0,
            ecx: 0,
            eax: 0,
            gs: 0,
            fs: SegmentSelector::UserDataSelector as u16,
            es: SegmentSelector::UserDataSelector as u16,
            ds: SegmentSelector::UserDataSelector as u16,
            eip: fun_addr,
            cs: SegmentSelector::UserCodeSelector as u32,
            eflags: Self::compose_default_eflags().get_data(),
            esp: user_stack_addr,
            ss: SegmentSelector::UserDataSelector as u32,
        }
    }

    pub fn init(&mut self, fun_addr: u32, user_stack_addr: u32) {
        self.edi = 0;
        self.esi = 0;
        self.ebp = 0;
        self.esp_dummy = 0;
        self.ebx = 0;
        self.edx = 0;
        self.ecx = 0;
        self.eax = 0;
        self.gs = 0;
        self.fs = SegmentSelector::UserDataSelector as u16;
        self.es = SegmentSelector::UserDataSelector as u16;
        self.ds = SegmentSelector::UserDataSelector as u16;
        self.eip = fun_addr;
        self.cs = SegmentSelector::UserCodeSelector as u32;
        self.eflags = Self::compose_default_eflags().get_data();
        self.esp = user_stack_addr;
        self.ss = SegmentSelector::UserDataSelector as u32;
    }

    /**
     * 构建默认的eflags寄存器的值
     */
    fn compose_default_eflags() -> EFlags {
        let mut empty_eflags = reg_eflags::EFlags::load();
        // let mut empty_eflags = reg_eflags::EFlags::empty();
        empty_eflags.set_off(FlagEnum::InputOutputPrivilegeLevel);
        // 保留位，为1
        empty_eflags.set_on(FlagEnum::FirstReserved);
        // 中断位，打开，设置1
        empty_eflags.set_on(FlagEnum::InterruptFlag);
        empty_eflags
    }
}

impl Display for InterruptStack {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        console_println!("InterruptStack(edi:0x{:x}, esi:0x{:x}, ebp:0x{:x}, esp_dummy:0x{:x}, ebx:0x{:x}, edx:0x{:x}, ecx:0x{:x}, eax:0x{:x}, gs:0x{:x}, fs:0x{:x}, es:0x{:x}, ds:0x{:x}, eip:0x{:x}, cs:0x{:x}, eflags:0x{:x}, esp:0x{:x}, ss:0x{:x})",
        self.edi as u32, 
        self.esi as u32, 
        self.ebp as u32, 
        self.esp_dummy as u32, 
        self.ebx as u32, 
        self.edx as u32, 
        self.ecx as u32, 
        self.eax as u32, 
        self.gs as u32,
        self.fs as u32, 
        self.es as u32, 
        self.ds as u32, 
        self.eip as u32,
        self.cs as u32, 
        self.eflags as u32, 
        self.esp as u32, 
        self.ss as u32);
        Result::Ok(())
    }
}


/**
 * 唤醒某一个线程。
 * 某个阻塞的线程只能被其他线程唤醒
 */
#[inline(never)]
pub fn wake_thread(task: &mut TaskStruct)  {
    // 关闭中断
    let old_status = instruction::disable_interrupt();
    // 只能是这三种状态之一
    let allow_status = [TaskStatus::TaskBlocked, TaskStatus::TaskHanging, TaskStatus::TaskWaiting];
    if !allow_status.contains(&task.task_status) {
        MY_PANIC!("could not wake up thread; name:{}, status:{:?}", task.get_name(), &task.task_status);
    }

    ASSERT!(task.task_status != TaskStatus::TaskReady);
    // 设置为就绪状态
    task.task_status = TaskStatus::TaskReady;
    // 放入就绪队列
    self::append_read_thread(task);

    // 恢复中断
    instruction::set_interrupt(old_status);
}