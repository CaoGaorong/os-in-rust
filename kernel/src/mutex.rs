use core::{cell::UnsafeCell, ops::{Deref, DerefMut}};

use os_in_rust_common::println;

use crate::sync;

/**
 * 实现一个阻塞的锁
 */

pub struct Mutex<T: ?Sized> {
    lock: sync::Lock,
    data: UnsafeCell<T>,
}
impl <T> Mutex<T> {
    pub fn new(data: T) -> Self {
        Self {
            lock: sync::Lock::new(),
            data: UnsafeCell::new(data),
        }
    }

    pub fn init(&mut self) {
        self.lock.init();
    }

    pub fn lock(&mut self) -> MutexGuard<T>{
        // 阻塞的锁
        self.lock.lock();
        // 返回的MutexGuard
        MutexGuard {
            lock: &mut self.lock,
            data: unsafe { &mut *self.data.get() },
        }
    }
}
unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}
unsafe impl<T: ?Sized + Send> Send for Mutex<T> {}



pub struct MutexGuard<'a, T: ?Sized + 'a> {
    lock: &'a mut sync::Lock,
    data: &'a mut T,
}

impl <'a, T:?Sized> Deref for MutexGuard<'a, T> {
    type Target = T;

    /**
     * 解引用。
     * 指定返回值的生命周期跟MutexGuard生命周期一致。
     * 这样避免了编译器的自动推断，更加准确
     */
    fn deref<'b>(&'b self) -> &'b Self::Target {
        &*self.data
    }
}

impl <'a, T:?Sized> DerefMut for MutexGuard<'a, T> {
    fn deref_mut<'b>(&'b mut self) -> &'b mut Self::Target {
        &mut *self.data
    }
}

impl <'a, T: ?Sized> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        println!("drop trait");
        // drop的时候解锁
        self.lock.unlock();
    }
}