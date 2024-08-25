use crate::thread::{self, TaskStruct};
use crate::println;

/**
 * ps命令的效果
 */
pub fn ps() {
    println!("PID  PPID    STAT    TICKS  LEFT_TICKS  TASK_NAME ");
    let all_thread_iter = thread::get_all_thread().iter();
    for task_node in all_thread_iter {
        let task = unsafe {&*TaskStruct::parse_by_all_tag(&*task_node)};
        println!("{:^3}  {:^5} {:^8} {:^6} {:^12} {:^12}", task.pid.get_data(), task.parent_pid.map_or(0, |pid| pid.get_data()), task.task_status.get_name(), task.elapsed_ticks, task.left_ticks, task.get_name());
    }
}