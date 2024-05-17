use os_in_rust_common::queue::{ArrayQueue, Queue};

/**
 * 实现一个阻塞队列
 */
pub struct BlockingQueue<T: Copy> {
    // 里面放一个普通的队列
    queue: Queue<T>,
}

