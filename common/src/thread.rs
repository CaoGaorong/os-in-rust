use core::{ptr, task};

use crate::{constants, memory};

/**
 * 关于线程的实现
 */

pub type ThreadFunc = fn (*const u8);

/** 
 * PCB的实现
*/
pub struct TaskStruct {
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
     * PCB内核栈地址
     */
    kernel_stack_ptr: *mut u32,
    /**
     * 栈边界的魔数
     */
    stack_magic: u32,
}

impl TaskStruct {
    pub fn new(name: &'static str, priority: u8, function: ThreadFunc, arg: *const u8) -> Self {
        let pcb = Self {
            name,
            priority,
            task_status: TaskStatus::TaskRunning,
            kernel_stack_ptr: ptr::null_mut(),
            stack_magic: constants::TASK_STRUCT_STACK_MAGIC,
        };
        pcb.kernel_stack_ptr = ((&pcb as *const _ as u32) + constants::PAGE_SIZE) as *mut u32;



    }
}

pub enum TaskStatus {
    TaskRunning,
    TaskReady,
    TaskBlocked,
    TaskWaiting,
    TaskHanging,
    TaskDied
}

pub struct ThreadStack {
    ebp: u32,
    ebx: u32,
    edi: u32,
    esi: u32,
    eip: fn (ThreadFunc, *const u8),
    ret_addr: *const u8,
    function: ThreadFunc,
    func_arg: *const u8
}