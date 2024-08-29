use crate::{pid_allocator::Pid, scheduler, thread::{self, TaskStatus, TaskStruct}, thread_management};

use super::TaskExitStatus;

/**
 * 父进程等待子进程，使用wait等待，然后给子进程“收尸”
 */
#[inline(never)]
pub fn wait() -> Option<(Pid, Option<TaskExitStatus>)> {
    let cur_task = &mut thread::current_thread().task_struct;
    
    loop {
        // 找已经结束的子进程
        let hanging_child = self::find_hanging_child(cur_task.pid);
        if hanging_child.is_some() {
            let hanging_child = hanging_child.unwrap();
            // 子进程的信息
            let child_exit_status = hanging_child.exit_status;
            let child_pid = hanging_child.pid;

            // 把child给回收掉（收尸）
            thread_management::free_thread(hanging_child);

            return Option::Some((child_pid, child_exit_status));
        }

        // 如果连子进程都没有，结束
        let child = cur_task.find_child();
        if child.is_none() {
            return Option::None;
        }

        // 阻塞当前线程
        scheduler::block_thread(cur_task, TaskStatus::TaskWaiting);
    }
}


#[inline(never)]
fn find_hanging_child(pid: Pid) -> Option<&'static mut TaskStruct> {
    // 遍历所有任务
    for tag in thread::get_all_thread().iter() {
        let task = unsafe {&mut *TaskStruct::parse_by_all_tag(&*tag)};
        let task_parent_id = task.parent_pid;
        if task_parent_id.is_some() && task_parent_id.unwrap() == pid && TaskStatus::TaskHanging == task.task_status {
            return Option::Some(task);
        }
    }
    return Option::None;
}
