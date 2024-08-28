use crate::filesystem::FileDescriptorType;
use crate::blocking_queue::ArrayBlockingQueue;
use crate::{memory, thread};

use super::pipe_holder::{PipeError, PipeReader, PipeWriter};
use super::pipe_container::{self, PipeContainer};

/**
 * 创建管道
 */
#[inline(never)]
pub fn pipe(size: usize) -> Result<(PipeReader, PipeWriter), PipeError> {
    // 申请一个数组，底层的缓冲区结构
    let buff = unsafe { core::slice::from_raw_parts_mut(memory::sys_malloc(size * size_of::<u8>()) as *mut u8, size) };
    // 一个阻塞队列结构
    let blocking_queue = ArrayBlockingQueue::new(buff);
    
    // 把管道，安装。得到下标
    let idx = pipe_container::install_pipe(PipeContainer::<u8>::new(blocking_queue));
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

    Result::Ok((PipeReader::new(fd), PipeWriter::new(fd)))
}