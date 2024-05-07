use core::{arch::asm, mem::size_of, ptr};

use crate::{constants, memory, ASSERT};

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
    pcb_page
        .task_struct
        .clone_from(&TaskStruct::new(thread_name, priority));

    // 填充PCB页的中断栈
    pcb_page
        .thread_stack
        .clone_from(&ThreadStack::new(func, arg));

    // 重置该页，PCB的栈指针的位置
    pcb_page.reset_stack_pointer();


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
     * 重置栈的指针
     * 把它重置到线程栈的开始位置处
     */
    pub fn reset_stack_pointer(&mut self) {
        self.task_struct.kernel_stack_ptr = &mut self.thread_stack as *mut ThreadStack as *mut u8;
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
}

impl TaskStruct {
    pub fn new(name: &'static str, priority: u8) -> Self {
        Self {
            kernel_stack_ptr: ptr::null_mut(),
            name,
            priority,
            task_status: TaskStatus::TaskRunning,
            stack_magic: constants::TASK_STRUCT_STACK_MAGIC,
        }
    }

    pub fn init_stack(&mut self, stack_addr: *mut u8) {
        self.kernel_stack_ptr = stack_addr;
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
    pub fn new(function: ThreadFunc, arg: ThreadArg) -> Self {
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
}

#[repr(C, packed)]
pub struct InterruptStack {}
impl InterruptStack {
    pub fn new() -> Self {
        Self {}
    }
}


