use core::{arch::asm, mem::size_of, ptr};

use os_in_rust_common::{constants, elem2entry, instruction::enable_interrupt, linked_list::LinkedNode, paging::{self, PageTable}, pool::MemPool, reg_cr3::{self, CR3}};

use crate::page_util;


/**
 * 创建一个内核线程，传递的函数
 */
pub type ThreadFunc = fn(ThreadArg);

/**
 * 内核线程，函数执行的参数
 */
pub type ThreadArg = u32;



extern "C" fn kernel_thread(function: ThreadFunc, arg: ThreadArg) {
    // 开启中断。线程切换依赖时钟中断
    enable_interrupt();
    function(arg)
}


/**
 * 获取当前运行的线程
 */
pub fn current_thread() -> &'static mut PcbPage {
    let cur_esp: u32;
    unsafe {
        asm!(
            "mov {0:e}, esp",
            out(reg) cur_esp,
        );
    }
    unsafe { &mut *((cur_esp & 0xfffff000) as *mut PcbPage) }
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
    thread_stack: ThreadStack,
    /**
     * 一页的部分最后是中断栈
     */
    interrupt_stack: InterruptStack,
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
    pub fn init_task_struct(&mut self, name: &'static str, priority: u8) {
        // 线程栈的地址
        let thread_stack_ptr = &mut self.thread_stack as *mut ThreadStack as u32;
        // 初始化任务信息
        self.task_struct.init(name, priority, thread_stack_ptr);
    }

    /**
     * 初始化线程栈
     */
    pub fn init_thread_stack(&mut self, function: ThreadFunc, arg: ThreadArg) {
        self.thread_stack.init(function, arg);
    }

    /**
     * 加载PCB页。
     * 把把“上下文”恢复到当前CPU寄存器中，利用ret指令，把(esp + 4)的值赋值到eip寄存器，来实现跳转执行
     */
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
}
/**
 * PCB的结构
*/
#[repr(C)]
pub struct TaskStruct {
    /**
     * PCB内核栈地址
     */
    pub kernel_stack: u32,
    /**
     * PCB的名称
     */
    pub name: &'static str,
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
     * 栈边界的魔数
     */
    pub stack_magic: u32,
}

impl TaskStruct {
    pub fn new(name: &'static str, priority: u8) -> Self {
        Self {
            kernel_stack: 0,
            name,
            priority,
            task_status: TaskStatus::TaskReady,
            left_ticks: priority,
            elapsed_ticks: 0,
            pgdir: ptr::null_mut(),
            general_tag: LinkedNode::new(),
            all_tag: LinkedNode::new(),
            vaddr_pool: MemPool::empty(),
            stack_magic: constants::TASK_STRUCT_STACK_MAGIC,
        }
    }

    fn init(&mut self, name: &'static str, priority: u8, kernel_stack: u32) {
        self.kernel_stack = kernel_stack;
        self.name = name;
        self.task_status = TaskStatus::TaskReady;
        self.priority = priority;
        self.stack_magic = constants::TASK_STRUCT_STACK_MAGIC;
        self.left_ticks = priority;
        self.elapsed_ticks = 0;
        self.pgdir = ptr::null_mut();
        self.general_tag = LinkedNode::new();
        self.all_tag = LinkedNode::new();
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

    /**
     * 激活该任务的页表
     */
    pub fn activate_pgdir(&self) {
        let page_dir_phy_addr: u32;
        // 如果该任务没有页表
        if self.pgdir == ptr::null_mut() {
            // 取内核的页表
            page_dir_phy_addr = paging::get_dir_ref();
        } else {
            // 取自己的页表，转成物理地址
            page_dir_phy_addr = page_util::get_phy_from_virtual_addr(self.pgdir as u32);
        }

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

}

// #[repr(u32)]
#[derive(Clone, Copy, PartialEq)]
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
 */
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct ThreadStack {
    /*---C语言ABI的标准———— */
    ebp: u32,
    ebx: u32,
    edi: u32,
    esi: u32,
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
    fn new(function: ThreadFunc, arg: ThreadArg) -> Self {
        Self {
            ebp: 0,
            ebx: 0,
            edi: 0,
            esi: 0,
            eip: kernel_thread,
            ret_addr: ptr::null(), // 占位用，没啥用
            function,
            func_arg: arg,
        }
    }
    
    fn init(&mut self, function: ThreadFunc, arg: ThreadArg) {
        self.ebp = 0;
        self.ebx = 0;
        self.edi = 0;
        self.esi = 0;
        self.eip = kernel_thread;
        self.ret_addr = ptr::null();
        self.function = function;
        self.func_arg = arg;
    }
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct InterruptStack {
    
}
impl InterruptStack {
    pub fn new() -> Self {
        Self {}
    }
}


