use core::arch::asm;

use os_in_rust_common::{elem2entry, instruction, reg_cr0::CR0, reg_eflags, thread::{self, TaskStatus, TaskStruct}, ASSERT};

use crate::{interrupt, thread_management};



pub fn schedule() {
    // 确保没有被打断
    ASSERT!(!instruction::is_intr_on());

    let cur_thread = thread::current_thread();
    let cur_task = &cur_thread.task_struct;
    // 原本该线程处于正在运行，那么说明是时间中断，定时切换的
    if cur_task.task_status == TaskStatus::TaskRunning {
        // 确保不在就绪队列中
        ASSERT!(!thread_management::get_ready_thread().contains(&cur_task.ready_tag));
        // 把当前线程加入到就绪队列
        thread_management::get_ready_thread().append(&mut cur_task.ready_tag);
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
    let task_to_run = unsafe { &*(TaskStruct::parse_by_ready_tag(pcb_ready_tag)) };
    task_to_run.set_status(TaskStatus::TaskRunning);
    
    // 从当前的任务，切换到要运行的任务
    switch_to(cur_task, task_to_run);
}

fn switch_to(cur_task: &TaskStruct, task_to_run: &TaskStruct) {
    
    todo!()
}
