use crate::{pid_allocator::Pid, scheduler, thread::{self, TaskStatus}, thread_management};

use super::TaskExitStatus;

/**
 * 父进程等待子进程，使用wait等待，然后给子进程“收尸”
 */
#[inline(never)]
pub fn wait() -> Option<(Pid, Option<TaskExitStatus>)> {
    let cur_task = &mut thread::current_thread().task_struct;
    // 找到子进程
    let child = cur_task.find_child();
    // 没有子进程
    if child.is_none() {
        return Option::None;
    }
    
    let child = child.unwrap();
    // 如果子进程没有执行完，那么阻塞当前任务
    if child.task_status != TaskStatus::TaskHanging {
        scheduler::block_thread(cur_task, TaskStatus::TaskWaiting);
    }
    // 子进程的信息
    let child_exit_status = child.exit_status;
    let child_pid = child.pid;

    // 把child给回收掉（收尸）
    thread_management::free_thread(child);

    return Option::Some((child_pid, child_exit_status));
}

