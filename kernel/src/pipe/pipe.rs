use crate::filesystem::{FileDescriptor, FileDescriptorType};
use crate::blocking_queue::ArrayBlockingQueue;
use crate::{memory, thread};

use super::pipe_holder::PipeError;
use super::pipe_container::{self, PipeContainer};

/**
 * 创建管道
 */
#[inline(never)]
pub fn pipe(size: usize) -> Result<FileDescriptor, PipeError> {
    // 申请一个数组，底层的缓冲区结构
    let buff_addr = memory::malloc_system(size * size_of::<u8>());
    let buff = unsafe { core::slice::from_raw_parts_mut(buff_addr as *mut u8, size) };
    
    // 把缓冲区安装到管道，得到管道数组的下标
    let idx = pipe_container::install_pipe(PipeContainer::<u8>::new(ArrayBlockingQueue::new(buff)));
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