use core::arch::asm;

use os_in_rust_common::{constants, instruction, ASSERT};

use crate::{thread::{self, TaskStatus, TaskStruct}, thread_management};



/**
 * 检查任务的调度
 */
pub fn check_task_schedule() {
    // 当前任务
    let current_thread = thread::current_thread();
    // 确保栈没有溢出
    ASSERT!(current_thread.task_struct.stack_magic == constants::TASK_STRUCT_STACK_MAGIC);
    let task_struct = &mut current_thread.task_struct;

    // 该进程运行的tick数+1
    task_struct.elapsed_ticks += 1;

    // 如果剩余的时间片还有，那就减少
    if task_struct.left_ticks > 0 {
        task_struct.left_ticks -= 1;
    } else {
        // 否则就切换其他线程
        schedule();
    }
}


#[inline(never)]
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
        thread_management::append_read_thread(cur_task);
        // 重置剩余的ticks
        cur_task.reset_ticks();
        // 设置为就绪
        cur_task.set_status(TaskStatus::TaskReady);
    // 如果是其他原因，可能是阻塞了
    } else {

    }
    // 如果没有就绪任务，那么就执行idle线程
    if thread_management::get_ready_thread().is_empty() {
        let idle_thread = thread_management::get_idle_thread();
        thread_management::wake_thread(idle_thread);
    }
    ASSERT!(!thread_management::get_ready_thread().is_empty());
    let pcb_ready_tag = thread_management::get_ready_thread().pop();
    // 找到那个要运行的task
    let task_to_run = unsafe { &mut *(TaskStruct::parse_by_general_tag(pcb_ready_tag)) };
    
    task_to_run.set_status(TaskStatus::TaskRunning);

    // 当前是内核程序，更换页表之前，可以使用输出语句
    // console_println!("cur task:(addr:0x{:x}, name:{}, status:{:?})", cur_task as *const _ as usize, cur_task.name as &str, cur_task.task_status);
    // console_println!("to task:(addr:0x{:x}, name:{}, status:{:?})",task_to_run as *const _ as usize, task_to_run.name as &str, task_to_run.task_status);

    // 激活这个进程
    task_to_run.activate_process();

    // printkln!("switch from:{}, to:{}", cur_task.name as &str, task_to_run.name as &str);

    // 从当前的任务，切换到要运行的任务j
    switch_to(cur_task, task_to_run);

}

/**
 * 切换任务（该函数不可用。该方法已经使用纯汇编来实现）
 * - cur_task: 当前任务
 * - task_to_run: 待运行的任务
 *      该方法的效果，本来是：保存当前任务的上下文，恢复下一个任务的上下文。
 *      但是在实际情况中，却没法正常运行。
 *      主要是switch_to函数的返回语句，这个rust实现的switch_to，经过反汇编，发现返回语句使用的是 iret指令。但是当前并不希望使用iret指令
 */

 #[inline(never)]
extern "C" fn switch_to_wrapper(cur_task: &mut TaskStruct, task_to_run: &mut TaskStruct) {
    switch_to(cur_task, task_to_run)
}

/**
 * 线程切换
 * #[no_mangle]和#[inline(never)]两个缺一不可
 * - #[inline(never)]：如果不使用这个，那么switch_to函数就不会是以函数调用的形式调用，而是可能直接内联到上游了
 * - #[no_mangle]：如果不使用这个，switch_to会被优化成jmp指令跳转，而不是call指令。jmp指令就不存在把eip压栈了
 */
#[no_mangle]
#[inline(never)]
#[cfg(all(not(test), target_arch = "x86"))]
extern "C" fn switch_to(cur_task: &mut TaskStruct, task_to_run: &mut TaskStruct) {

    // 保存上下文。callee-saved register
    // 如果不要线程栈的这几个寄存器，就可以不用保存了
    // unsafe {
    //     asm!(
    //         "push esi",
    //         "push edi",
    //         "push ebx",
    //         "push ebp",
    //     )
    // }
    // 保存上下文（把当前esp的值保存到当前任务）
    let mut cur_esp: u32;
    unsafe {
        asm!(
            "mov {:e}, esp",
            out(reg) cur_esp
        )
    }
    cur_task.kernel_stack = cur_esp;

    // 恢复上下文，把下一个任务的esp的值给恢复
    let next_esp = task_to_run.kernel_stack;
    unsafe {
        asm!(
            "mov esp, {:e}",
            in(reg) next_esp
        )
    }

    // 恢复上下文
    // unsafe {
    //     asm!(
    //         "pop ebp",
    //         "pop ebx",
    //         "pop edi",
    //         "pop esi"
    //     )
    // }
}

#[cfg(all(not(target_arch = "x86")))]
extern "C" fn switch_to(cur_task: &mut TaskStruct, task_to_run: &mut TaskStruct) {
    todo!()
}
