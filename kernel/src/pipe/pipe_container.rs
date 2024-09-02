
use os_in_rust_common::racy_cell::RacyCell;
use crate::{blocking_queue::{ArrayBlockingQueue, BlockingQueue}, filesystem::{FileDescriptor, FileDescriptorType}, memory, thread::{self, TaskStruct}};


const PILE_LIST_SIZE: usize = 10;

/**
 * 管道列表。整个系统的管道，都放在这里
 */
const NONE_PIPE: Option<PipeContainer<'static, u8>> = Option::None;
static PIPE_LIST: RacyCell<[Option<PipeContainer<'static, u8>>; PILE_LIST_SIZE]> = RacyCell::new([NONE_PIPE; PILE_LIST_SIZE]);


#[inline(never)]
pub fn get_pipe_list() -> &'static mut [Option<PipeContainer<'static, u8>>] {
    unsafe { PIPE_LIST.get_mut() }
}

#[inline(never)]
pub fn get_pipe(idx: usize) -> &'static mut  Option<PipeContainer<'static, u8>> {
    &mut self::get_pipe_list()[idx]
}


#[inline(never)]
pub fn free_pipe(idx: usize) {
    *self::get_pipe(idx) = Option::None;
}



#[inline(never)]
pub fn install_pipe(blocking_queue: ArrayBlockingQueue<'static, u8>, task: &'static TaskStruct) -> Option<usize> {
    let pipe_list = self::get_pipe_list();
    for (idx, pipe) in pipe_list.iter_mut().enumerate() {
        if pipe.is_none() {
            *pipe = Option::Some(PipeContainer::<u8>::new(blocking_queue, task));
            return Option::Some(idx);
        }
    }
    return Option::None;
}

/**
 * 通过文件描述符或者
 */
#[inline(never)]
pub fn get_pipe_by_fd(fd: FileDescriptor) -> Option<&'static mut PipeContainer<'static, u8>> {
    let cur_task = &thread::current_thread().task_struct;
    let task_file_descriptor = cur_task.fd_table.get_task_file_descriptor(fd)?;
    if task_file_descriptor.get_fd_type() != FileDescriptorType::Pipe {
        return Option::None;
    }
    let global_idx = task_file_descriptor.get_global_idx();
    self::get_pipe(global_idx).as_mut()
}

pub fn release_pipe(fd: FileDescriptor) {
    let fd_table = &mut thread::current_thread().task_struct.fd_table;
    let task_file_descriptor = fd_table.get_task_file_descriptor(fd);
    if task_file_descriptor.is_none() {
        return;
    }
    if task_file_descriptor.unwrap().get_fd_type() != FileDescriptorType::Pipe {
        return;
    }
    // 空出当前任务的文件描述符
    fd_table.set_task_file_descriptor(fd, Option::None);

    // 释放全局管道
    self::free_pipe(task_file_descriptor.unwrap().get_global_idx());
    return;
}

/**
 * 管道的结构，底层是一个阻塞队列，可读可写
 */
pub struct PipeContainer<'a, T: Copy + Sized> {
    /**
     * 管道底层是一个阻塞队列
     */
    queue: ArrayBlockingQueue<'a, T>,
    /**
     * 这个管道的生产者。生产者任务退出时，需要往管道里结束
     */
    producer: &'a TaskStruct,
    /**
     * 这个管道的消费者。消费者退出时，会销毁管道
     */
    consumer: &'a TaskStruct,
}

impl <'a, T: Copy + Sized> PipeContainer<'a, T> {
    
    
    #[inline(never)]
    pub fn new(queue: ArrayBlockingQueue<'a, T>, creator: &'a TaskStruct) -> Self {
        Self {
            queue,
            producer: creator,
            consumer: creator,
        }
    }

    /**
     * 读取该管道的数据，读取一个元素出来。阻塞操作
     */
    #[inline(never)]
    pub fn read(&mut self, buff: &mut [T]) -> usize {
        for (idx, ele) in buff.iter_mut().enumerate() {
            // 从队列里面取数据（阻塞）
            let data = self.queue.take();
            // 如果为空了，那么这个队列彻底空了
            if data.is_none() {
                return idx;
            }
            *ele = data.unwrap();
        }
        return buff.len();
    }

    /**
     * 写入该管道的数据，写入一个元素。阻塞操作
     */
    #[inline(never)]
    pub fn write(&mut self, data: &[T]) {
        for ele in data {
            self.queue.put(*ele)
        }
    }

    #[inline(never)]
    pub fn write_end(&mut self) {
        self.queue.end();
    }

    #[inline(never)]
    pub fn get_producer(&self) -> &TaskStruct {
        &self.producer
    }


    #[inline(never)]
    pub fn get_consumer(&self) -> &TaskStruct {
        &self.consumer
    }

    #[inline(never)]
    pub fn set_producer(&mut self, producer: &'a TaskStruct) {
        self.producer = producer;
    }

    #[inline(never)]
    pub fn set_consumer(&mut self, consumer: &'a TaskStruct) {
        self.consumer = consumer;
    }
    
}

/**
 * 管道drop掉后，释放掉内存
 */
impl <'a, T: Copy + Sized> Drop for PipeContainer<'a, T> {
    #[inline(never)]
    fn drop(&mut self) {
        memory::free_system(self.queue.get_data().as_ptr());
    }
}