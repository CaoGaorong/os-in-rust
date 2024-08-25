use core::arch::asm;

use os_in_rust_common::{constants, cstr_write, instruction, utils, ASSERT};

use crate::{common::exec_dto::ExecParam, filesystem::{self}, interrupt, memory, thread};

#[derive(Debug)]
pub enum ExecError {
    Init,
    OpenFileError(filesystem::FileError),
}

const USER_PROC_ENTRY_ADDR: usize = 0xc048000;

#[inline(never)]
pub fn execv(param: &ExecParam) -> Result<(), ExecError> {
    
    // 把这个文件，加载到0xc048000
    self::load(param.get_file_path(), USER_PROC_ENTRY_ADDR)?;

    let cur_pcb = thread::current_thread();
    cstr_write!(cur_pcb.task_struct.get_name_mut(), "{}", param.get_file_path());

    let intr_stack = &mut cur_pcb.interrupt_stack;
    // 这个文件的起始地址，就是执行入口
    intr_stack.init_exec(USER_PROC_ENTRY_ADDR.try_into().unwrap(), param.get_args());

    let intr_stack_addr = intr_stack as *const _ as u32;
    cur_pcb.task_struct.kernel_stack = intr_stack_addr;

    // 设置esp的值，为这个地址
    instruction::set_esp(intr_stack_addr);
    // 中断退出
    interrupt::intr_exit();
    
    return Result::Ok(());
}

#[inline(never)]
fn load(file_path: &str, addr: usize) -> Result<(), ExecError> {
    // 打开文件
    let exec_file = filesystem::File::open(file_path);
    if exec_file.is_err() {
        return Result::Err(ExecError::OpenFileError(exec_file.unwrap_err()));
    }
    let exec_file = exec_file.unwrap();
    
    let cur_pcb = thread::current_thread();
    
    let file_size = exec_file.get_size().unwrap();
    let page_cnt = utils::div_ceil(file_size as u32, constants::PAGE_SIZE) as usize;
    
    for page_idx in 0..page_cnt {

        let cur_addr = addr + page_idx * constants::PAGE_SIZE as usize;
        
        // 申请1页内存
        memory::malloc_user_page_by_vaddr(&mut cur_pcb.task_struct.vaddr_pool, cur_addr);
        
        // 把这个页空间转成一个buffer
        let buff =  unsafe { core::slice::from_raw_parts_mut(cur_addr as *mut u8, constants::PAGE_SIZE as usize) };
        
        // 读取文件到指定的内存空间中
        let read_res = exec_file.read(buff);
        if read_res.is_err() {
            return Result::Err(ExecError::OpenFileError(read_res.unwrap_err()));
        }
    }
    return Result::Ok(());
}