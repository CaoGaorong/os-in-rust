use core::sync::atomic::AtomicBool;

use os_in_rust_common::queue::{ArrayQueue, Queue, QueueError};

use crate::sync::Semaphore;


/**
 * 实现一个阻塞队列。trait
 */
pub trait BlockingQueue<T: Copy + Sized>: Queue<T> {
    /**
     * 放一个元素，阻塞
     */
    fn put(&mut self, ele: T);
    /**
     * 取一个元素，阻塞
     */
    fn take(&mut self) -> Option<T>;

    /**
     * 结束阻塞。生产者调用这个方法，可以结束阻塞（消费者消费完后不会阻塞）
     */
    fn end(&mut self);

    /**
     * 获取到这个队列内的数据信息
     */
    fn get_data(&mut self) -> &mut [T];
}
/**
 * 使用数组实现一个阻塞队列
 */
pub struct ArrayBlockingQueue<'a, T: Copy + Sized> {
    /**
     * 存放元素的普通队列。使用锁保证互斥
     */
    queue: ArrayQueue<'a, T>,
    /**
     * 该队列的生产者信号量。生产者阻塞在这里
     */
    producer: Semaphore,
    /**
     * 队列的消费者信号量。消费者阻塞在这里
     */
    consumer: Semaphore,

    /**
     * 是否结束生成
     */
    end: AtomicBool, 
}

impl <'a, T: Copy + Sized> ArrayBlockingQueue<'a, T> {
    /**
     * 构造一个数组阻塞队列
     */
    #[inline(never)]
    pub const fn new(data: &'a mut [T]) -> Self {
        Self {
            producer: Semaphore::new(data.len() as u32),
            consumer: Semaphore::new(0),
            queue: ArrayQueue::new(data),
            end: AtomicBool::new(false),
        }
    }

    /**
     * 这个阻塞队列已经终止
     */
    pub fn end(&mut self) {
        self.end.store(true, core::sync::atomic::Ordering::Release);
    }
}

impl <'a, T: Copy + Sized> Queue<T> for ArrayBlockingQueue<'a, T> {
    fn append(&mut self, data: T) -> Result<bool, QueueError> {
        self.queue.append(data)
    }

    fn poll(&mut self) -> Option<T> {
        self.queue.poll()
    }

    fn size(&self) -> u32 {
        self.queue.size()
    }

    fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    fn is_full(&self) -> bool {
        self.queue.is_full()
    }
}

impl <'a, T: Copy + Sized> BlockingQueue<T> for ArrayBlockingQueue<'a, T> {
    /**
     * 往阻塞队列里面放队列。
     * 队列满就会阻塞
     */
    fn put(&mut self, ele: T) {
        // 生产者拿走一个信号量，阻塞生产者
        self.producer.down();
        // 添加元素
        let _ = self.queue.append(ele);
        // 添加元素后，还给消费者一个信号量，通知消费者可以消费了
        self.consumer.up();
    }

    /**
     * 从阻塞队列里面取出元素。
     * 返回值：
     *      - Option::Some(T)：从阻塞队列取出的数据，没有数据则阻塞
     *      - Option::None：队列里已经空了，并且队列不阻塞了（不再生产数据了）
     */
    #[inline(never)]
    fn take(&mut self) -> Option<T> {
        // 如果已经结束了，无需阻塞消费者
        if !self.end.load(core::sync::atomic::Ordering::Acquire) {
            // 一定要先阻塞 消费者
            self.consumer.down();
        }
        // 再取出元素
        let ele = self.queue.poll();
        // 取出元素后，通知生产者可以继续放了
        self.producer.up();
        
        ele
    }
    
    /**
     * 生产者结束生产
     */
    fn end(&mut self) {
        // 生产者拿走一个信号量，阻塞生产者
        self.producer.down();
        self.end.store(true, core::sync::atomic::Ordering::Release);
        // 添加元素后，还给消费者一个信号量，通知消费者可以消费了
        self.consumer.up();
    }


    fn get_data(&mut self) -> &mut [T] {
        self.queue.get_data()
    }
    

}

