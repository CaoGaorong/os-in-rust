use core::arch::asm;

use os_in_rust_common::{cstr_write, ASSERT};

use crate::{filesystem, interrupt, memory, thread};

#[derive(Debug)]
pub enum ExecError {
    Init,
    OpenFileError(filesystem::FileError),
}

#[inline(never)]
pub fn execv(path: &str) -> Result<(), ExecError> {
    // 打开文件
    let exec_file = filesystem::File::open(path);
    if exec_file.is_err() {
        return Result::Err(ExecError::OpenFileError(exec_file.unwrap_err()));
    }
    let exec_file = exec_file.unwrap();
    let file_size = exec_file.get_size();
    ASSERT!(file_size.is_ok());
    let file_size = file_size.unwrap();

    // 创建一个buffer
    let buff =  unsafe { core::slice::from_raw_parts_mut(memory::sys_malloc(file_size) as *mut u8, file_size) };


    // 读取这个文件
    let read_res = exec_file.read(buff);
    if read_res.is_err() {
        return Result::Err(ExecError::OpenFileError(read_res.unwrap_err()));
    }

    let cur_pcb = thread::current_thread();
    cstr_write!(cur_pcb.task_struct.get_name_mut(), "{}", path);

    let intr_stack = &mut cur_pcb.interrupt_stack;
    // 这个文件的起始地址，就是执行入口
    intr_stack.init_exec(buff.as_ptr() as u32);

    let intr_stack_addr = intr_stack as *const _ as u32;
    cur_pcb.task_struct.kernel_stack = intr_stack_addr;

    unsafe {
        asm!(
            "mov esp, {0:e}",
            in(reg) intr_stack_addr
        );
    }
    interrupt::intr_exit();
    
    return Result::Ok(());
}