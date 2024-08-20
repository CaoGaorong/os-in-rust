use core::{fmt, mem::size_of, str};

use os_in_rust_common::{printkln, vga, ASSERT, MY_PANIC};

use crate::{blocking_queue::BlockingQueue, console, filesystem::{self, DirError, FileDescriptor}, fork, keyboard, memory, scancode::KeyCode, thread, thread_management};
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
fn write(fd: u32, addr: u32, len: u32) -> u32 {

    let buf = unsafe { core::slice::from_raw_parts_mut(addr as *mut u8, len.try_into().unwrap()) };

    let fd = FileDescriptor::new(fd.try_into().unwrap());
    if filesystem::StdFileDescriptor::StdOutputNo as usize == fd.get_value() {
        let str_res = str::from_utf8(buf);
        ASSERT!(str_res.is_ok());
        let string = str_res.unwrap();
        printkln!("{}", string);
        return string.len() as u32;
    }

    
    let file = filesystem::get_file_by_fd(fd).unwrap();
    let fs = filesystem::get_filesystem();
    filesystem::write_file(fs, file, buf).try_into().unwrap()
}

/**
 * read系统调用
 */
#[inline(never)]
fn read(fd: u32, buf: u32, len: u32) -> u32 {
    let fd = FileDescriptor::new(fd.try_into().unwrap());
    let buf = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, len.try_into().unwrap()) };
    // 如果是标准输入
    if filesystem::StdFileDescriptor::StdInputNo as usize == fd.get_value() {
        let key_buff = unsafe { core::slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut KeyCode, buf.len() / size_of::<KeyCode>()) };
        let mut idx = 0;
        // 键盘队列
        let keyboard_queue = keyboard::get_keycode_queue();
        while idx < key_buff.len() {
            // 逐个取出输入的键，直到满了
            let key_res = keyboard_queue.take();
            if key_res.is_none() {
                continue;
            }
            key_buff[idx] = key_res.unwrap();
            idx += 1;
        }
        return idx.try_into().unwrap();
    }

    // 根据文件描述符，得到这个文件
    let file = filesystem::get_file_by_fd(fd).unwrap();
    let fs = filesystem::get_filesystem();
    // 读取文件
    filesystem::read_file(fs, file, buf).try_into().unwrap()
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
