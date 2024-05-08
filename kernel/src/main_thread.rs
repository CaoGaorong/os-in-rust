use os_in_rust_common::{constants, thread::{self, TaskStruct}};

pub fn make_thread_main() {
    let pcb_page  = thread::current_thread();
    pcb_page.init_task_struct(constants::MAIN_THREAD_NAME, 31);
    pcb_page.
    
}