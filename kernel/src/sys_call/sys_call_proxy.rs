use core::arch::asm;

use core::fmt;
use core::mem::size_of_val;

use crate::common::cwd_dto::CwdDto;
use crate::common::exec_dto::ExecParam;
use crate::exec;
use crate::filesystem::{self, FileDescriptor, SeekFrom, StdFileDescriptor};
use crate::pid_allocator::Pid;
use crate::pipe::{PipeError, PipeReader, PipeWriter};
use crate::scancode::KeyCode;
use crate::userprog::TaskExitStatus;

use super::sys_call::SystemCallNo;

/**
 * 本模块理论上是系统库提供给用户进程使用了，不是操作系统的实现提供的。
 * 比如gcc库给C用户程序提供的发起系统调用的包
 */


/**
 * 系统调用 print!
 */
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::sys_call::sys_print(format_args!($($arg)*)));
}

/**
 * 系统调用 println!
 */
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

/**
 * 获取当前任务的pid
 */
#[inline(never)]
pub fn get_pid() -> Pid {
    Pid::new(do_sys_call(SystemCallNo::GetPid, Option::None, Option::None, Option::None).try_into().unwrap())
}


/**
 * 给定Arguments，然后发起系统调用
 */
#[no_mangle]
pub fn sys_print(args: fmt::Arguments) {
    // 取出参数的地址
    let arg_addr = &args as *const _ as u32;
    // 调用系统调用，把参数地址传过去
    do_sys_call(SystemCallNo::Print, Option::Some(arg_addr), Option::None, Option::None);
}

/**
 * 写入字符
 */
pub fn write(fd: FileDescriptor, buff: &[u8]) {
    do_sys_call(SystemCallNo::Write, Option::Some(&fd as *const _ as u32), Option::Some(buff.as_ptr() as u32), Option::Some(buff.len().try_into().unwrap()));
}

/**
 * 发起系统调用，申请bytes大小的内存空间
 */
pub fn malloc<T>(bytes: usize) -> &'static mut T {
    let addr = do_sys_call(SystemCallNo::Malloc, Option::Some(bytes as u32), Option::None, Option::None);
    unsafe { &mut *(addr as *mut T) }
}

/**
 * 发起系统调用，释放内存空间
 */
pub fn free<T>(ptr: *const T) {
    let addr = ptr as u32;
    do_sys_call(SystemCallNo::Free, Option::Some(addr as u32), Option::None, Option::None);
}

pub enum ForkResult {
    Parent(Pid),
    Child
}

#[inline(never)]
pub fn fork() -> ForkResult {
    // 调用系统调用的fork
    let fork_res = self::do_sys_call(SystemCallNo::Fork, Option::None, Option::None, Option::None);
    if fork_res == 0 {
        ForkResult::Child
    } else {
        ForkResult::Parent(Pid::new(fork_res.try_into().unwrap()))
    }
}


/**
 * 线程挂起
 */
pub fn thread_yield() {
    self::do_sys_call(SystemCallNo::Yield, Option::None, Option::None, Option::None);
}

/**
 * 清除屏幕
 */
pub fn clear_screen() {
    self::do_sys_call(SystemCallNo::ClearScreen, Option::None, Option::None, Option::None);
}


#[inline(never)]
pub fn read_key() -> KeyCode {
    let mut key = KeyCode::empty();
    let buf = unsafe { core::slice::from_raw_parts_mut(&mut key as *mut _ as *mut u8, size_of_val(&key)) };
    self::read(FileDescriptor::new(StdFileDescriptor::StdInputNo as usize), buf);
    key
}

pub fn read(fd: FileDescriptor, buff: &mut[u8]) -> usize {
    self::do_sys_call(SystemCallNo::Read, Option::Some(&fd as *const _ as u32), Option::Some(buff.as_mut_ptr() as u32), Option::Some(buff.len() as u32)) as usize
}


#[inline(never)]
pub fn read_dir(path: &str) -> Result<filesystem::ReadDir, filesystem::DirError> {
    let mut res: Result<filesystem::ReadDir, filesystem::DirError> = Result::Err(filesystem::DirError::AlreadyExists);
    self::do_sys_call(SystemCallNo::ReadDir, Option::Some(path.as_ptr() as u32), Option::Some(path.len() as u32), Option::Some(&mut res as *mut _ as u32));
    return res;
}

#[inline(never)]
pub fn remove_dir(path: &str) -> Result<(), filesystem::DirError> {
    let mut res: Result<(), filesystem::DirError> = Result::Ok(());
    self::do_sys_call(SystemCallNo::RemoveDir, Option::Some(path.as_ptr() as u32), Option::Some(path.len() as u32), Option::Some(&mut res as *mut _ as u32));
    return res;
}

#[inline(never)]
pub fn create_dir_all(path: &str) -> Result<(), filesystem::DirError> {
    let mut res: Result<(), filesystem::DirError> = Result::Ok(());
    self::do_sys_call(SystemCallNo::CreateDirAll, Option::Some(path.as_ptr() as u32), Option::Some(path.len() as u32), Option::Some(&mut res as *mut _ as u32));
    return res;
}

#[inline(never)]
pub fn create_dir(path: &str) -> Result<(), filesystem::DirError> {
    let mut res: Result<(), filesystem::DirError> = Result::Ok(());
    self::do_sys_call(SystemCallNo::CreateDir, Option::Some(path.as_ptr() as u32), Option::Some(path.len() as u32), Option::Some(&mut res as *mut _ as u32));
    return res;
}

#[inline(never)]
pub fn dir_iter<'a>(dir: &'a mut filesystem::ReadDir) -> Option<filesystem::ReadDirIterator<'a>> {
    let mut res: Option<filesystem::ReadDirIterator<'a>> = Option::None;
    self::do_sys_call(SystemCallNo::DirIterator, Option::Some(dir as *mut  filesystem::ReadDir as u32), Option::Some(&mut res as *mut Option<filesystem::ReadDirIterator> as u32), Option::None);
    res
}

#[inline(never)]
pub fn dir_iter_next(iter: &mut  filesystem::ReadDirIterator) -> Option<&'static filesystem::DirEntry> {
    let mut res: Option<&filesystem::DirEntry> = Option::None;
    self::do_sys_call(SystemCallNo::DirIteratorNext, Option::Some(iter as *mut _ as u32), Option::Some(&mut res as *mut _ as u32), Option::None);
    return res;
}

#[inline(never)]
pub fn dir_iter_drop(iter: &mut  filesystem::ReadDirIterator) {
    self::do_sys_call(SystemCallNo::DirIteratorDrop, Option::Some(iter as *mut _ as u32), Option::None, Option::None);
}


#[inline(never)]
pub fn create_file(path: &str) -> Result<filesystem::File, filesystem::FileError> {
    let mut res: Result<filesystem::File, filesystem::FileError> = Result::Err(filesystem::FileError::NotFound);
    self::do_sys_call(SystemCallNo::CreateFile, Option::Some(path.as_ptr() as u32), Option::Some(path.len() as u32), Option::Some(&mut res as *mut _ as u32));
    res
}

#[inline(never)]
pub fn open_file(path: &str) -> Result<filesystem::File, filesystem::FileError> {
    let mut res: Result<filesystem::File, filesystem::FileError> = Result::Err(filesystem::FileError::NotFound);
    self::do_sys_call(SystemCallNo::OpenFile, Option::Some(path.as_ptr() as u32), Option::Some(path.len() as u32), Option::Some(&mut res as *mut _ as u32));
    res
}

/**
 * seek
 */
#[inline(never)]
pub fn seek_file(file: &mut filesystem::File, seek: SeekFrom) -> Result<(), filesystem::FileError> {
    let mut res: Result<(), filesystem::FileError> = Result::Err(filesystem::FileError::NotFound);
    self::do_sys_call(SystemCallNo::Seek, Option::Some(file as *mut _ as u32), Option::Some(&seek as *const _ as u32), Option::Some(&mut res as *mut _ as u32));
    res
}

#[inline(never)]
pub fn close_file(file: &mut filesystem::File) -> Result<(), filesystem::FileError> {
    let mut res: Result<(), filesystem::FileError> = Result::Err(filesystem::FileError::NotFound);
    self::do_sys_call(SystemCallNo::CloseFile, Option::Some(file as *mut _ as u32), Option::Some(&mut res as *mut _ as u32), Option::None);
    res
}

#[inline(never)]
pub fn file_size(file: &filesystem::File) -> Result<usize, filesystem::FileError> {
    let mut res: Result<usize, filesystem::FileError> = Result::Err(filesystem::FileError::NotFound);
    self::do_sys_call(SystemCallNo::FileSize, Option::Some(file as *const _ as u32), Option::Some(&mut res as *mut _ as u32), Option::None);
    res
}

#[inline(never)]
pub fn remove_file(path: &str) -> Result<(), filesystem::FileError> {
    let mut res: Result<(), filesystem::FileError> = Result::Err(filesystem::FileError::NotFound);
    self::do_sys_call(SystemCallNo::RemoveFile, Option::Some(path.as_ptr() as u32), Option::Some(path.len() as u32), Option::Some(&mut res as *mut _ as u32));
    res
}


#[inline(never)]
pub fn exec(param: &ExecParam) -> Result<(), exec::ExecError> {
    let mut res: Result<(), exec::ExecError> = Result::Err(exec::ExecError::Init);
    self::do_sys_call(SystemCallNo::Exec, Option::Some(param as *const _ as u32), Option::Some(&mut res as *mut _ as u32), Option::None);
    return res;
}


#[inline(never)]
pub fn exit(status: TaskExitStatus) {
    self::do_sys_call(SystemCallNo::Exit, Option::Some(status as u32), Option::None, Option::None);
}

#[inline(never)]
pub fn wait() -> Option<(Pid, Option<u8>)> {
    let mut res: Option<(Pid, Option<u8>)> = Option::None;
    self::do_sys_call(SystemCallNo::Wait, Option::Some(&mut res as *mut _ as u32), Option::None,  Option::None);
    res
}


#[inline(never)]
pub fn get_cwd(path: &mut[u8]) -> &str {
    let mut dto = CwdDto {
        buff: path,
        str: Option::None,
    };
    self::do_sys_call(SystemCallNo::Cwd, Option::Some(&mut dto as *mut _ as u32), Option::None, Option::None);
    dto.str.unwrap()
}

#[inline(never)]
pub fn change_dir(path: &str) -> Option<()> {
    let mut res: Option<()> = Option::None;
    self::do_sys_call(SystemCallNo::Cd, Option::Some(path.as_ptr() as u32), Option::Some(path.len() as u32), Option::Some(&mut res as *mut _ as u32));
    res
}

#[inline(never)]
pub fn pipe(size: usize) -> Result<(PipeReader, PipeWriter), PipeError> {
    let mut res: Result<(PipeReader, PipeWriter), PipeError> = Result::Err(PipeError::PipeExhaust);
    self::do_sys_call(SystemCallNo::PipeCreate, Option::Some(size as u32), Option::Some(&mut res as *mut _ as u32), Option::None);
    res
}


#[inline(never)]
pub fn pipe_end(fd: FileDescriptor) {
    self::do_sys_call(SystemCallNo::PipeEnd, Option::Some(fd.get_value() as u32), Option::None, Option::None);
}

/**
 * 发起系统调用
 * eax: 系统调用号
 * ebx_opt：第一个参数；可选
 * ecx_opt：第二个参数；可选
 * edx_opt：第三个参数；可选
 */
#[inline(always)]
#[cfg(all(not(test), target_arch = "x86"))]
fn do_sys_call(eax_sys_nr: SystemCallNo, ebx_opt: Option<u32>, ecx_opt: Option<u32>, edx_opt: Option<u32>) -> u32 {
    let eax = eax_sys_nr as u32;
    let res: u32;
    unsafe {
        asm!("mov eax, eax", in("eax") eax);
        if ebx_opt.is_some() {
            let ebx = ebx_opt.unwrap();
            asm!("mov ebx, ebx", in("ebx")ebx );
        }
        if ecx_opt.is_some() {
            let ecx = ecx_opt.unwrap();
            asm!("mov ecx, ecx", in("ecx")ecx );
        }
        if edx_opt.is_some() {
            let edx = edx_opt.unwrap();
            asm!("mov edx, edx", in("edx")edx );
        }
        asm!(
            "int 0x80",
            "mov eax, eax",
            out("eax") res,
        );
    }
    res
}
#[cfg(all(not(target_arch = "x86")))]
fn do_sys_call(eax_sys_nr: SystemCallNo, ebx_opt: Option<u32>, ecx_opt: Option<u32>, edx_opt: Option<u32>) -> u32 {
    todo!()
}