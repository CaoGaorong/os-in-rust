
use os_in_rust_common::racy_cell::RacyCell;
use crate::{blocking_queue::{ArrayBlockingQueue, BlockingQueue}, filesystem::{FileDescriptor, FileDescriptorType}, memory, thread};
use crate::println;


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
pub fn install_pipe(container: PipeContainer<'static, u8>) -> Option<usize> {
    let pipe_list = self::get_pipe_list();
    for (idx, pipe) in pipe_list.iter_mut().enumerate() {
        if pipe.is_none() {
            *pipe = Option::Some(container);
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


/**
 * 管道的结构，底层是一个阻塞队列，可读可写
 */
pub struct PipeContainer<'a, T: Copy + Sized> {
    queue: ArrayBlockingQueue<'a, T>,
}

impl <'a, T: Copy + Sized> PipeContainer<'a, T> {
    #[inline(never)]
    pub fn new(queue: ArrayBlockingQueue<'a, T>) -> Self {
        Self {
            queue
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

    pub fn write_end(&mut self) {
        self.queue.end();
    }
    
}

/**
 * 管道drop掉后，释放掉内存
 */
impl <'a, T: Copy + Sized> Drop for PipeContainer<'a, T> {
    fn drop(&mut self) {
        // println!("pipe drop");
        memory::sys_free(self.queue.get_data().as_ptr() as usize);
    }
}