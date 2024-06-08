use core::ptr;


use os_in_rust_common::{instruction, linked_list::LinkedList, printkln, ASSERT};

use crate::{console_println, thread::{self, TaskStruct}, thread_management};

/**
 * 定义一个信号量
 *      - 设计一个信号量，就是一个信号量的值，以及等待该信号量的任务队列
 *      - 成功获取到信号量(value > 0)，则把value -= 1，直接成功
 *      - 如果信号量的值已经不够用(value <= 0)，那么当前线程就会进入阻塞队列（等待持有信号量的线程释放信号量后来唤醒），切换到其他就绪线程来执行
 */
pub struct Semaphore {
    /**
     * 信号量的值
     */
    pub value: u32,
    /**
     * 等待进程的列表
     */
    pub waiters: LinkedList,
}
impl Semaphore {
    /**
     * 创建一个有初始化值的信号量
     */
    pub const fn new(value: u32) -> Self {
        let mut linked_list = LinkedList::new();
        Self {
            value,
            waiters: linked_list,
        }
    }
    /**
     * 初始化
     */
    // pub fn init(&mut self) {
    //     self.waiters.init();
    // }
    /**
     * 信号量减少操作。**阻塞操作**
     */
    pub fn down(&mut self) {
        // let old_status = instruction::disable_interrupt();

        
        // 信号量的值小于等于0，需要每次都阻塞，然后被唤醒后重新判断信号量的值
        while self.value <= 0 {
            let current_thread = &mut thread::current_thread().task_struct;
            ASSERT!(!self.waiters.contains(&current_thread.general_tag));
            // 把自己放入该信号量的等待队列
            self.waiters.append(&mut current_thread.general_tag);
            thread_management::block_thread(current_thread, thread::TaskStatus::TaskBlocked);
        }
        // 把信号量减一
        self.value -= 1;
        // instruction::set_interrupt(old_status);
    }

    /**
     * 是信号量增加。会唤醒等待的线程（并不会立马执行唤醒的线程，而是只是加到就绪队列）
     */
    pub fn up(&mut self) {
        let old_status = instruction::disable_interrupt();
        // print!("{:?}", old_status);
        // println!("interrupt on:{}", instruction::is_intr_on());
        // println!("thread:{}, interrupt status old:{:?}", thread::current_thread().task_struct.name, old_status);
        // 如果没任务等待这个信号量
        if self.waiters.is_empty() {
            self.value += 1;
            instruction::set_interrupt(old_status);
            return;
        }
        // 从等待队列中，找出一个等待者
        let waiting_task_tag = self.waiters.pop();
        let waiting_task = unsafe { &mut *TaskStruct::parse_by_general_tag(waiting_task_tag) };
        // 唤醒这个等待者
        thread_management::wake_thread(waiting_task);
        // 增加信号量
        self.value += 1;
        instruction::set_interrupt(old_status);
    }
}

/**
 * 定义一个锁的结构。本质上就是一个二元信号量，加锁和解锁
 *  设计一个锁，有一下几部分：
 *      - semaphore: 该锁的信号量，锁就是2元信号量。
 *      - holder: 锁的持有者。为了实现可重入锁
 *      - repeat: 锁重入次数。为了实现可重入锁
 */
pub struct Lock {
    /**
     * 信号量。锁本质是一个二元信号量。lock和unlock
     */
    semaphore: Semaphore,
    /**
     * 持有该所的线程。为了实现可重入锁
     */
    holder: *mut TaskStruct,
    /**
     * 重复申请锁的次数。为了实现可重入锁
     */
    repeat: u32,
}

unsafe impl Send for Lock {}
unsafe impl Sync for Lock {}



impl Lock {
    /**
     * 创建一个持有者为holder的锁
     */
    pub const fn new() -> Self {
        Self {
            holder: ptr::null_mut(),
            // 二元信号量
            semaphore: Semaphore::new(1),
            repeat: 0,
        }
    }

    // pub fn init(&mut self) {
    //     self.semaphore.init();
    // }
    /**
     * 加锁。阻塞操作
     * 如果加锁成功（未进入阻塞或者阻塞后退出），那么当前线程变为锁的持有者
     */
    pub fn lock(&mut self) {
        let current_task = &mut thread::current_thread().task_struct;
        // 如果是当前任务锁的持有者，说明重入了
        if self.holder as u32 == current_task as *mut _ as u32 {
            self.repeat += 1;
            return;
        }
        // 信号量减1
        self.semaphore.down();
        // 当前任务为该锁的持有者。能运行到这里，那么说明必定没有阻塞
        self.holder = current_task as *mut _;
        self.repeat += 0;
    }

    /**
     * 释放锁。释放后会唤醒其他等待的线程
     */
    pub fn unlock(&mut self) {
        let current_task = &thread::current_thread().task_struct;
        // println!("cur:{}, holder:{}", current_task.name, unsafe {&*self.holder}.name);
        // println!("holder:{}", self.holder as u32);
        ASSERT!(current_task as *const _ as u32 == self.holder as u32);
        if self.repeat > 1 {
            self.repeat -= 1;
            return;
        }
        // 该锁没有持有者
        self.holder = ptr::null_mut();
        // 没人用
        self.repeat = 0;
        // 唤醒等待该信号量的线程
        self.semaphore.up();
    }
}