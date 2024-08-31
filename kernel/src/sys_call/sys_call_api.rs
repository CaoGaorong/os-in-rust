use core::{fmt, mem::{size_of, take}, str, task};

use os_in_rust_common::{vga::{self}, ASSERT, MY_PANIC};

use crate::{blocking_queue::BlockingQueue, common::{cwd_dto::CwdDto, exec_dto::ExecParam}, console, console_print, exec, filesystem::{self, DirError, FileDescriptor, FileDescriptorType}, fork, keyboard, memory, pid_allocator::Pid, pipe::{self, PipeError, PipeReader, PipeWriter}, scancode::KeyCode, thread, thread_management, userprog::{self, TaskExitStatus}};
use super::sys_call::{self, HandlerType, SystemCallNo};

/**
 * 这里是系统调用暴露出去的接口
 */

/**
 * 初始化系统调用接口
 */
#[inline(never)]
pub fn init() {
    // getPid
    sys_call::register_handler(SystemCallNo::GetPid, HandlerType::NoneParam(get_pid));

    // write
    sys_call::register_handler(SystemCallNo::Write, HandlerType::ThreeParams(write));
    
    // Read
    sys_call::register_handler(SystemCallNo::Read, HandlerType::ThreeParams(read));

    // println
    sys_call::register_handler(SystemCallNo::Print, HandlerType::OneParam(print_format));
    
    // malloc
    sys_call::register_handler(SystemCallNo::Malloc, HandlerType::OneParam(malloc));
    
    // free
    sys_call::register_handler(SystemCallNo::Free, HandlerType::OneParam(free));
    
    // fork
    sys_call::register_handler(SystemCallNo::Fork, HandlerType::NoneParam(fork));
    
    // yield
    sys_call::register_handler(SystemCallNo::Yield, HandlerType::NoneParam(thread_yield));
    
    // 清除屏幕
    sys_call::register_handler(SystemCallNo::ClearScreen, HandlerType::NoneParam(clear_screen));
    
    // 读取目录
    sys_call::register_handler(SystemCallNo::ReadDir, HandlerType::ThreeParams(read_dir));
    
    // 删除目录
    sys_call::register_handler(SystemCallNo::RemoveDir, HandlerType::ThreeParams(remove_dir));

    // 创建目录
    sys_call::register_handler(SystemCallNo::CreateDir, HandlerType::ThreeParams(create_dir));

    // 递归创建目录
    sys_call::register_handler(SystemCallNo::CreateDirAll, HandlerType::ThreeParams(create_dir_all));
    
    // 生成目录迭代器
    sys_call::register_handler(SystemCallNo::DirIterator, HandlerType::TwoParams(dir_iter));
    
    // 目录迭代器
    sys_call::register_handler(SystemCallNo::DirIteratorNext, HandlerType::TwoParams(dir_iter_next));
    
    // 目录迭代器drop
    sys_call::register_handler(SystemCallNo::DirIteratorDrop, HandlerType::OneParam(dir_iter_drop));
    
    // 创建文件
    sys_call::register_handler(SystemCallNo::CreateFile, HandlerType::ThreeParams(create_file));

    // 打开文件
    sys_call::register_handler(SystemCallNo::OpenFile, HandlerType::ThreeParams(open_file));
    
    // 获取文件大小
    sys_call::register_handler(SystemCallNo::FileSize, HandlerType::TwoParams(file_size));

    // 关闭文件
    sys_call::register_handler(SystemCallNo::CloseFile, HandlerType::TwoParams(close_file));
    
    // 删除文件
    sys_call::register_handler(SystemCallNo::RemoveFile, HandlerType::ThreeParams(remove_file));
    
    // exec
    sys_call::register_handler(SystemCallNo::Exec, HandlerType::TwoParams(exec));
    
    // exit
    sys_call::register_handler(SystemCallNo::Exit, HandlerType::OneParam(exit));
    
    // wait
    sys_call::register_handler(SystemCallNo::Wait, HandlerType::OneParam(wait));

    // cwd
    sys_call::register_handler(SystemCallNo::Cwd, HandlerType::OneParam(get_cwd));
    
    // cd
    sys_call::register_handler(SystemCallNo::Cd, HandlerType::ThreeParams(change_dir));
    
    // 创建管道
    sys_call::register_handler(SystemCallNo::PipeCreate, HandlerType::TwoParams(pipe_create));
    
    // 关闭管道
    sys_call::register_handler(SystemCallNo::PipeEnd, HandlerType::OneParam(pipe_end));
    
    // 重定向当前任务的某个文件描述符
    sys_call::register_handler(SystemCallNo::FileDescriptorRedirect, HandlerType::TwoParams(redirect_file_descriptor));
}

/**
 * 获取当前任务的pid
 */
fn get_pid() -> u32 {
    let cur_task = &thread::current_thread().task_struct;
    cur_task.pid.get_data().try_into().unwrap()
}

/**
 * write系统调用
 */
#[inline(never)]
fn write(fd_addr: u32, addr: u32, len: u32) -> u32 {
    let fd  = unsafe { *(fd_addr as *const FileDescriptor) };
    let buf = unsafe { core::slice::from_raw_parts_mut(addr as *mut u8, len.try_into().unwrap()) };

    // 根据文件描述符找到
    let task_file_descriptor = filesystem::get_task_file_descriptor(fd);
    if task_file_descriptor.is_none() {
        return 0;
    }
    let task_file_descriptor = task_file_descriptor.unwrap();

    // 如果是控制台
    if task_file_descriptor.get_fd_type() == FileDescriptorType::Console {
        // 如果是标准输出，那么就打印到控制台
        if filesystem::StdFileDescriptor::StdOutputNo as usize == task_file_descriptor.get_global_idx() {
            let str_res = str::from_utf8(buf);
            ASSERT!(str_res.is_ok());
            let string = str_res.unwrap();
            console_print!("{}", string);
            return string.len() as u32;
        }
        return 0;
    }
    
    // 如果是管道
    if task_file_descriptor.get_fd_type() == FileDescriptorType::Pipe {
        let pipe_container = pipe::get_pipe_by_fd(fd);
        if pipe_container.is_none() {
            return 0;
        }
        let pipe_container = pipe_container.unwrap();
        pipe_container.write(buf);
        return 0;
    }

    // 普通文件
    if task_file_descriptor.get_fd_type() == FileDescriptorType::File {
        let file = filesystem::get_file_by_fd(fd).unwrap();
        let fs = filesystem::get_filesystem();
        return filesystem::write_file(fs, file, buf).try_into().unwrap()
    }
    return 0;
}

/**
 * read系统调用
 */
#[inline(never)]
fn read(fd_addr: u32, buf: u32, len: u32) -> u32 {
    let fd  = unsafe { *(fd_addr as *const FileDescriptor) };
    let buf = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, len.try_into().unwrap()) };
    let task_file_descriptor = filesystem::get_task_file_descriptor(fd);
    ASSERT!(task_file_descriptor.is_some());
    let task_file_descriptor = task_file_descriptor.unwrap();

    // 如果是控制台操作
    if task_file_descriptor.get_fd_type() == FileDescriptorType::Console {
        // 如果是标准输入
        if filesystem::StdFileDescriptor::StdInputNo as usize == fd.get_value() {
            let key_buff = unsafe { core::slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut KeyCode, buf.len() / size_of::<KeyCode>()) };
            let mut idx = 0;
            // 键盘队列
            let keyboard_queue = keyboard::get_keycode_queue();
            while idx < key_buff.len() {
                // 逐个取出输入的键，直到满了
                let key_res = keyboard_queue.take().unwrap();
                if key_res.is_none() {
                    continue;
                }
                key_buff[idx] = key_res.unwrap();
                idx += 1;
            }
            return idx.try_into().unwrap();
        }
        return 0;
    }

    // 如果是管道
    if task_file_descriptor.get_fd_type() == FileDescriptorType::Pipe {
        // 根据文件描述符，找到管道
        let pipe_container = pipe::get_pipe_by_fd(fd);
        ASSERT!(pipe_container.is_some());
        let pipe_container = pipe_container.unwrap();

        // 从管道里读取出数据
        return pipe_container.read(buf).try_into().unwrap();
    }

    // 普通文件
    if task_file_descriptor.get_fd_type() == FileDescriptorType::File {
        // 根据文件描述符，得到这个文件
        let file = filesystem::get_file_by_fd(fd).unwrap();
        let fs = filesystem::get_filesystem();
        // 读取文件
        return filesystem::read_file(fs, file, buf).try_into().unwrap();
    }
    return 0;
}

/**
 * 打印系统调用
 */
#[inline(never)]
fn print_format(argument_addr: u32) -> u32 {
    // 把参数地址，转成对象
    let arg = unsafe { *(argument_addr as *const fmt::Arguments) };
    
    // 使用console_print函数，打印
    console::console_print(arg);
    0
}

/**
 * 申请bytes大小的内存空间
 */
fn malloc(bytes: u32) -> u32 {
    memory::sys_malloc(bytes as usize) as u32
}

/**
 * 释放某个地址的内存空间
 */
fn free(addr_to_free: u32) -> u32 {
    memory::sys_free(addr_to_free.try_into().unwrap());
    0
}


/**
 * fork
 */
fn fork() -> u32 {
    fork::fork().get_data().try_into().unwrap()
}

fn thread_yield() -> u32 {
    thread_management::thread_yield();
    0
}

/**
 * 清除屏幕
 */
fn clear_screen() -> u32 {
    vga::clear_all();
    0
}

/**
 * 读取目录
 */
#[inline(never)]
fn read_dir(addr: u32, len: u32, dir_addr: u32) -> u32 {
    let dir = dir_addr as *mut Result<filesystem::ReadDir, DirError>;
    let dir_path = unsafe { core::str::from_utf8(core::slice::from_raw_parts(addr as *const u8, len.try_into().unwrap())) };
    ASSERT!(dir_path.is_ok());
    unsafe { *dir = filesystem::read_dir(dir_path.unwrap()) };
    0
}

/**
 * 删除目录
 */
#[inline(never)]
fn remove_dir(addr: u32, len: u32, res_addr: u32) -> u32 {
    let res = res_addr as *mut Result<(), DirError>;
    let dir_path = unsafe { core::str::from_utf8(core::slice::from_raw_parts(addr as *const u8, len.try_into().unwrap())) };
    ASSERT!(dir_path.is_ok());
    unsafe { *res = filesystem::remove_dir(dir_path.unwrap()) };
    0
}

#[inline(never)]
fn create_dir(addr: u32, len: u32, res_addr: u32) -> u32 {
    let res = res_addr as *mut Result<(), DirError>;
    let dir_path = unsafe { core::str::from_utf8(core::slice::from_raw_parts(addr as *const u8, len.try_into().unwrap())) };
    ASSERT!(dir_path.is_ok());
    unsafe { *res = filesystem::create_dir(dir_path.unwrap()) };
    0
}

#[inline(never)]
fn create_dir_all(addr: u32, len: u32, res_addr: u32) -> u32 {
    let res = res_addr as *mut Result<(), DirError>;
    let dir_path = unsafe { core::str::from_utf8(core::slice::from_raw_parts(addr as *const u8, len.try_into().unwrap())) };
    ASSERT!(dir_path.is_ok());
    unsafe { *res = filesystem::create_dir_all(dir_path.unwrap()) };
    0
}


#[inline(never)]
fn dir_iter(dir_addr: u32, res_addr: u32) -> u32 {
    let dir = unsafe { &mut *(dir_addr as *mut filesystem::ReadDir) };
    let res = unsafe { &mut *(res_addr as *mut Option<filesystem::ReadDirIterator>) };
    *res = Option::Some(dir.iter_ignore_drop());
    0
}

#[inline(never)]
fn dir_iter_next(iter_addr: u32, res_addr: u32) -> u32 {
    let iter = unsafe { &mut *(iter_addr as *mut filesystem::ReadDirIterator) };
    let res = unsafe { &mut *(res_addr as *mut Option<&filesystem::DirEntry>) };
    *res = iter.next();
    0
}

#[inline(never)]
fn dir_iter_drop(iter_addr: u32) -> u32 {
    let iter = unsafe { &mut *(iter_addr as *mut filesystem::ReadDirIterator) };
    // 手动触发drop
    iter.drop();
    0
}


fn create_file(addr: u32, len: u32, res_addr: u32) -> u32 {
    let res = unsafe {&mut *(res_addr as *mut Result<filesystem::File, filesystem::FileError>)};
    let dir_path = unsafe { core::str::from_utf8(core::slice::from_raw_parts(addr as *const u8, len.try_into().unwrap())) };
    ASSERT!(dir_path.is_ok());
    *res = filesystem::File::create_ignore_drop(dir_path.unwrap());
    0
}

#[inline(never)]
fn open_file(addr: u32, len: u32, res_addr: u32) -> u32 {
    let res = unsafe {&mut *(res_addr as *mut Result<filesystem::File, filesystem::FileError>)};
    let dir_path = unsafe { core::str::from_utf8(core::slice::from_raw_parts(addr as *const u8, len.try_into().unwrap())) };
    if dir_path.is_err() {
        MY_PANIC!("file error: {:?}", dir_path.unwrap_err());
    }
    *res = filesystem::File::open_ignore_drop(dir_path.unwrap());
    0
}

#[inline(never)]
fn file_size(file_addr: u32, res_addr: u32) -> u32 {
    let file = unsafe {&*(file_addr as *const filesystem::File)};
    let res = unsafe {&mut *(res_addr as *mut Result<usize, filesystem::FileError>)};
    *res = file.get_size();
    0
}


#[inline(never)]
fn seek_file(file_addr: u32, seek_addr: u32, res_addr: u32) -> u32 {
    let file = unsafe {&mut *(file_addr as *mut filesystem::File)};
    let res = unsafe { &mut *(res_addr as *mut Result<(), filesystem::FileError>) };
    let seek_from = unsafe { & *(seek_addr as *const filesystem::SeekFrom) };
    *res = file.seek(*seek_from);
    0
}


#[inline(never)]
fn close_file(file_addr: u32, res_addr: u32) -> u32 {
    let file = unsafe {&mut *(file_addr as *mut filesystem::File)};
    let res = unsafe {&mut *(res_addr as *mut Result<(), filesystem::FileError>)};
    *res = file.close();
    0
}


#[inline(never)]
fn remove_file(addr: u32, len: u32, res_addr: u32) -> u32 {
    let res = unsafe {&mut *(res_addr as *mut Result<(), filesystem::FileError>)};
    let dir_path = unsafe { core::str::from_utf8(core::slice::from_raw_parts(addr as *const u8, len.try_into().unwrap())) };
    ASSERT!(dir_path.is_ok());
    *res = filesystem::remove_file(dir_path.unwrap());
    0
}


#[inline(never)]
fn exec(param_addr: u32, res_addr: u32) -> u32 {
    let param = unsafe { &*(param_addr as *const ExecParam) };

    let res = unsafe {&mut *(res_addr as *mut Result<(), exec::ExecError>)};
    *res = exec::execv(param);

    return 0;
}

#[inline(never)]
fn exit(status: u32) -> u32 {
    userprog::exit(status as u8);
    0
}

#[inline(never)]
fn wait(res_addr: u32) -> u32 {
    let res = unsafe { &mut *(res_addr as *mut Option<(Pid, Option<TaskExitStatus>)>) };
    *res = userprog::wait();
    0
}

#[inline(never)]
fn get_cwd(dto_addr: u32) -> u32 {
    let cwd_dto = unsafe { &mut *(dto_addr as *mut CwdDto) };
    let cur_task = &thread::current_thread().task_struct;
    cwd_dto.str = filesystem::get_cwd(cur_task, cwd_dto.buff);
    return 0;
}

#[inline(never)]
fn change_dir(path_addr: u32, path_len: u32, res_addr: u32) -> u32 {
    let path = unsafe { core::str::from_utf8(core::slice::from_raw_parts(path_addr as *const u8, path_len.try_into().unwrap())) }.unwrap();
    let res = unsafe { &mut *(res_addr as *mut Option<()>) };
    let cur_task = &mut thread::current_thread().task_struct;
    *res = filesystem::change_dir(cur_task, path);
    0
}

#[inline(never)]
fn pipe_create(size: u32, res_addr: u32) -> u32 {
    let res = unsafe { &mut *(res_addr as *mut Result<FileDescriptor, PipeError>) };
    *res = pipe::pipe(size as usize);
    0
}

#[inline(never)]
fn pipe_end(fd_addr: u32) -> u32 {
    let fd = unsafe { *(fd_addr as *const FileDescriptor) };
    let task_fd = filesystem::get_task_file_descriptor(fd);
    if task_fd.is_none() {
        return 0;
    }
    // 如果不是管道，那么也不用
    if task_fd.unwrap().get_fd_type() != FileDescriptorType::Pipe {
        return 0;
    }
    // 找到管道，结束了
    let pipe = pipe::get_pipe_by_fd(fd);
    if pipe.is_none() {
        return 0;
    }
    pipe.unwrap().write_end();
    0
}

#[inline(never)]
fn redirect_file_descriptor(fd: u32, redirect_to: u32) -> u32 {
    let target_fd = unsafe { *(fd as *const FileDescriptor) };
    let redirect_to = unsafe { *(redirect_to as *const FileDescriptor) };
    
    filesystem::redirect_file_descriptor(target_fd, redirect_to);
    0
}