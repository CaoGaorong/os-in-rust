#![feature(global_asm)]
use core::{arch::{asm, global_asm}, ptr, task};

use os_in_rust_common::{elem2entry, instruction, print, println, reg_cr0::CR0, reg_eflags, ASSERT};

use crate::{console_println, interrupt, thread::{self, PcbPage, TaskStatus, TaskStruct}, thread_management};


global_asm!(include_str!("switch.s"));

extern "C" {
    fn switch_to(cur_task: &mut TaskStruct, task_to_run: &mut TaskStruct);
}

pub fn schedule() {

    // 确保没有被打断
    ASSERT!(!instruction::is_intr_on());

    let cur_thread = thread::current_thread();
    let cur_task = &mut cur_thread.task_struct;

    
    // 原本该线程处于正在运行，那么说明是时间中断，定时切换的
    if cur_task.task_status == TaskStatus::TaskRunning {
        // 确保不在就绪队列中
        ASSERT!(!thread_management::get_ready_thread().contains(&cur_task.general_tag));
        // 把当前线程加入到就绪队列
        thread_management::get_ready_thread().append(&mut cur_task.general_tag);
        // 重置剩余的ticks
        cur_task.reset_ticks();
        // 设置为就绪
        cur_task.set_status(TaskStatus::TaskReady);
    // 如果是其他原因，可能是阻塞了
    } else {

    }
    ASSERT!(!thread_management::get_ready_thread().is_empty());
    let pcb_ready_tag = thread_management::get_ready_thread().pop();
    // 找到那个要运行的task
    let task_to_run = unsafe { &mut *(TaskStruct::parse_by_general_tag(pcb_ready_tag)) };
    
    task_to_run.set_status(TaskStatus::TaskRunning);

    // 当前是内核程序，更换页表之前，可以使用输出语句
    if cur_task.pgdir == ptr::null_mut() {
        println!("switch from:{}, to:{}", cur_task.name as &str, task_to_run.name as &str);
    }

    // 激活这个进程
    task_to_run.activate_process();

    // 要之前的程序是内核程序，更换页表后，才可以使用输出语句
    if task_to_run.pgdir == ptr::null_mut() {
        println!("switch from:{}, to:{}", cur_task.name as &str, task_to_run.name as &str);
    }

    // 从当前的任务，切换到要运行的任务j
    unsafe { switch_to(cur_task, task_to_run) };

}

/**
 * 切换任务（该函数不可用。该方法已经使用纯汇编来实现）
 * - cur_task: 当前任务
 * - task_to_run: 待运行的任务
 *      该方法的效果，本来是：保存当前任务的上下文，恢复下一个任务的上下文。
 *      但是在实际情况中，却没法正常运行。
 *      主要是switch_to函数的返回语句，这个rust实现的switch_to，经过反汇编，发现返回语句使用的是 iret指令。但是当前并不希望使用iret指令
 */
#[no_mangle]
#[cfg(never)]
extern "C" fn switch_to(cur_task: &mut TaskStruct, task_to_run: &mut TaskStruct) {
    println!();
    println!("from:{}, to:{}", cur_task.name, task_to_run.name);
    println!("from: task addr:0x{:x}, task_stack_addr:0x{:x}", cur_task as *const TaskStruct as u32, cur_task.kernel_stack as u32);
    println!("to: task addr:0x{:x}, task_stack_addr:0x{:x}", task_to_run as *const TaskStruct as u32, task_to_run.kernel_stack as u32);

    unsafe {
        asm!(
            "push esi",
            "push edi",
            "push ebx",
            "push ebp",
        );
        let mut esp: u32;
        asm!(
            "mov {0:e}, esp",
            out(reg) esp
        );

        println!("current esp: 0x{:x}", esp);
        // 把目前esp的值，保存到当前任务的PCB中
        cur_task.kernel_stack = esp;

        // 待运行任务的上下文中的esp
        let new_esp = task_to_run.kernel_stack;
        // 恢复上下文
        asm!(
            "mov esp, {0:e}",
            in(reg) new_esp
        );

        asm!(
            "pop ebp",
            "pop ebx",
            "pop edi",
            "pop esi",
            "ret"
        );

    }
}
