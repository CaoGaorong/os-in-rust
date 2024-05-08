use core::{arch::asm, mem::size_of, ptr};

use crate::{constants, memory, paging::PageTable, println, ASSERT};

/**
 * 创建一个内核线程，传递的函数
 */
pub type ThreadFunc = fn(ThreadArg);

/**
 * 内核线程，函数执行的参数
 */
pub type ThreadArg = &'static str;


fn kernel_thread(function: ThreadFunc, arg: ThreadArg) {
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
 * 启动一个内核线程
 */
pub fn thread_start(thread_name: &'static str, priority: u8, func: ThreadFunc, arg: ThreadArg) {

    // 先确保，目前的PCB页大小构建正确。如果错误，请及时修复代码
    ASSERT!(size_of::<PcbPage>() as u32 == constants::PAGE_SIZE);

    // 申请1页内核内存
    let page_addr = memory::malloc_kernel_page(1);

    // 把内核内存转成PCB页
    let pcb_page = unsafe { &mut *(page_addr as *mut PcbPage) };

    // 构建PCB页
    pcb_page.init_task_struct(thread_name, priority);

    // 填充PCB页的中断栈
    pcb_page.init_thread_stack(func, arg);


    // 加载该PCB。加载到esp，然后使用ret触发
    pcb_page.do_load();
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
#[repr(C, packed)]
pub struct PcbPage {
    /**
     * 开始部分是PCB
     */
    task_struct: TaskStruct,
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
     * 初始化任务
     */
    pub fn init_task_struct(&mut self, name: &'static str, priority: u8) {
        // 初始化任务信息
        self.task_struct.init(name, priority);
        // 把栈指针放在线程栈地址处
        self.task_struct.kernel_stack_ptr = &mut self.thread_stack as *mut ThreadStack as *mut u8;
    }

    pub fn init_thread_stack(&mut self, function: ThreadFunc, arg: ThreadArg) {
        self.thread_stack.init(function, arg);
    }

    pub fn set_task_status(&mut self, status: TaskStatus) {
        self.task_struct.set_status(status);
    }

    /**
     * 加载PCB页。
     * 把把“上下文”恢复到当前CPU寄存器中，利用ret指令，把(esp + 4)的值赋值到eip寄存器，来实现跳转执行
     */
    pub fn do_load(&self) {
        // 当前PCB的栈指针指向的地址
        let stack_addr = self.task_struct.kernel_stack_ptr as u32;

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
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct TaskStruct {
    /**
     * PCB内核栈地址
     */
    kernel_stack_ptr: *mut u8,
    /**
     * PCB的名称
     */
    name: &'static str,
    /**
     * PCB状态
     */
    task_status: TaskStatus,
    /**
     * PCB优先级
     */
    priority: u8,
    /**
     * 栈边界的魔数
     */
    stack_magic: u32,

    /**
     * 当前进程/线程还剩的滴答数量
     */
    left_ticks: u8,
    /**
     * 该任务一共执行了多久
     */
    elapsed_ticks: u8,
    /**
     * 该PCB使用的页表地址
     */
    pgdir: *mut PageTable,
}

impl TaskStruct {
    pub fn new(name: &'static str, priority: u8) -> Self {
        Self {
            kernel_stack_ptr: ptr::null_mut(),
            name,
            priority,
            task_status: TaskStatus::TaskReady,
            stack_magic: constants::TASK_STRUCT_STACK_MAGIC,
            left_ticks: priority,
            elapsed_ticks: 0,
            pgdir: ptr::null_mut(),
        }
    }

    fn init(&mut self, name: &'static str, priority: u8) {
        self.kernel_stack_ptr = ptr::null_mut();
        self.name = name;
        self.task_status = TaskStatus::TaskReady;
        self.priority = priority;
        self.stack_magic = constants::TASK_STRUCT_STACK_MAGIC;
        self.left_ticks = priority;
        self.elapsed_ticks = 0;
        self.pgdir = ptr::null_mut();
    }

    fn set_status(&mut self , status: TaskStatus) {
        self.task_status = status;
    }

}

// #[repr(u32)]
#[derive(Clone, Copy)]
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
    eip: fn(ThreadFunc, ThreadArg),
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

#[repr(C, packed)]
pub struct InterruptStack {}
impl InterruptStack {
    pub fn new() -> Self {
        Self {}
    }
}


