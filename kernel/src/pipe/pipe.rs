use crate::filesystem::{self, FileDescriptor, FileDescriptorType, StdFileDescriptor};
use crate::blocking_queue::ArrayBlockingQueue;
use crate::{memory, thread};

use super::pipe_holder::{self, PipeError};
use super::pipe_container::{self, PipeContainer};


#[inline(never)]
pub fn set_consumer(pipe_fd: FileDescriptor) -> Result<(), PipeError> {
    
    // 如果是管道，那么要设置消费者
    let pipe = pipe_container::get_pipe_by_fd(pipe_fd);
    if pipe.is_some() {
        let consumer = &thread::current_thread().task_struct;
        let pipe = pipe.unwrap();
        pipe.set_consumer(consumer);
    }

    // 把该进程的标准输出，重定向到管道的输出
    filesystem::redirect_file_descriptor(FileDescriptor::new(StdFileDescriptor::StdInputNo as usize), pipe_fd);
    return Result::Ok(());
}


#[inline(never)]
pub fn set_producer(pipe_fd: FileDescriptor) -> Result<(), PipeError> {
    // 如果是管道，那么需要设置生产者
    let pipe = pipe_container::get_pipe_by_fd(pipe_fd);
    if pipe.is_some() {
        let pipe = pipe.unwrap();
        let producer = &thread::current_thread().task_struct;
        pipe.set_producer(producer);
    }

    // 把该进程的标准输出，重定向到管道的输出
    filesystem::redirect_file_descriptor(FileDescriptor::new(StdFileDescriptor::StdOutputNo as usize), pipe_fd);
    return Result::Ok(());
}


#[inline(never)]
pub fn release_pipe(fd: FileDescriptor) {
    pipe_container::release_pipe(fd);
}

/**
 * 创建管道
 */
#[inline(never)]
pub fn pipe(size: usize) -> Result<FileDescriptor, PipeError> {
    // 申请一个数组，底层的缓冲区结构
    let buff_addr = memory::malloc_system(size * size_of::<u8>());
    let buff = unsafe { core::slice::from_raw_parts_mut(buff_addr as *mut u8, size) };
    let cur_task = &mut thread::current_thread().task_struct;
    
    // 把缓冲区安装到管道，得到管道数组的下标
    let idx = pipe_container::install_pipe(ArrayBlockingQueue::new(buff), cur_task);
    if idx.is_none() {
        // 管道耗尽了
        return Result::Err(PipeError::PipeExhaust);
    }

    // 把管道的全局下标，安装到文件描述符中
    let cur_task = &mut thread::current_thread().task_struct;
    let fd = cur_task.fd_table.install_fd(idx.unwrap(), FileDescriptorType::Pipe);
    if fd.is_none() {
        // 这个任务的文件描述符耗尽了
        return Result::Err(PipeError::FileDescriptorExhaust);
    }
    let fd = fd.unwrap();
    Result::Ok(fd)

    // Result::Ok((PipeReader::new(fd), PipeWriter::new(fd)))
}
