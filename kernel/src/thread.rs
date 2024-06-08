use core::{arch::asm, fmt::{write, Display, Pointer}, mem::size_of, ptr};

use os_in_rust_common::{constants, elem2entry, instruction::{self, enable_interrupt}, linked_list::LinkedNode, paging::{self, PageTable}, pool::MemPool, printkln, reg_cr3::{self, CR3}, reg_eflags::{self, EFlags, FlagEnum}, selector::SegmentSelector};

use crate::{console_println, mem_block::{self, MemBlockAllocator}, page_util, pid_allocator, tss};


/**
 * 创建一个内核线程，传递的函数
 */
pub type ThreadFunc = extern "C" fn(ThreadArg);

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
    let cur_esp = instruction::load_esp();
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
    pub fn init_task_struct(&mut self, name: &'static str, priority: u8, pcb_page_addr: u32) {
        // 线程栈的地址
        let thread_stack_ptr = &mut self.thread_stack as *mut ThreadStack as u32;
        // 初始化任务信息
        self.task_struct.init(name, priority, thread_stack_ptr, pcb_page_addr);
    }

    /**
     * 初始化线程栈
     */
    pub fn init_thread_stack(&mut self, function: ThreadFunc, arg: ThreadArg) {
        self.thread_stack.init(function, arg);
    }

    pub fn init_intr_stack(&mut self, fun_addr: u32, user_stack_addr: u32) {
        self.interrupt_stack.init(fun_addr, user_stack_addr);
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
    pub pid: u8,
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
     * PCB页的地址
     */
    pub pcb_page_addr: u32,

    /**
     * 内存块分配器。支持分配多种规格的内存块
     */
    pub mem_block_allocator: MemBlockAllocator,

    /**
     * 栈边界的魔数
     */
    pub stack_magic: u32,
}

impl Display for TaskStruct {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        console_println!("TaskStruct(name:{}, kernel_stack:0x{:x}, task_status:{:?}, pgdir:0x{:x}, vaddr_pool:{})", self.name as &str, self.kernel_stack as u32, self.task_status, self.pgdir as u32, self.vaddr_pool);;
        Result::Ok(())
    }
}

impl TaskStruct {
    pub fn new(name: &'static str, priority: u8) -> Self {
        Self {
            pid: pid_allocator::allocate(),
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
            pcb_page_addr: 0,
            mem_block_allocator: MemBlockAllocator::new(),
        }
    }

    fn init(&mut self, name: &'static str, priority: u8, kernel_stack: u32, pcb_page_addr: u32) {
        self.pid = pid_allocator::allocate();
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
        self.pcb_page_addr = pcb_page_addr;
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
    fn activate_pgdir(&self) {
        let page_dir_phy_addr: usize;
        // 如果该任务没有页表
        if self.pgdir == ptr::null_mut() {
            // 取内核的页表物理地址。
            page_dir_phy_addr = constants::KERNEL_PAGE_DIR_ADDR;
        } else {
            // 取自己的页表，转成物理地址
            page_dir_phy_addr = page_util::get_phy_from_virtual_addr(self.pgdir as usize);
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
    fn new(function: ThreadFunc, arg: ThreadArg) -> Self {
        Self {
            // ebp: 0,
            // ebx: 0,
            // edi: 0,
            // esi: 0,
            eip: kernel_thread,
            ret_addr: ptr::null(), // 占位用，没啥用
            function,
            func_arg: arg,
        }
    }
    
    fn init(&mut self, function: ThreadFunc, arg: ThreadArg) {
        // self.ebp = 0;
        // self.ebx = 0;
        // self.edi = 0;
        // self.esi = 0;
        self.eip = kernel_thread;
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


