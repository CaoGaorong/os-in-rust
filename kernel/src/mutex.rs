use core::{cell::UnsafeCell, ops::{Deref, DerefMut}};

use crate::sync;

/**
 * 实现一个阻塞的锁
 * 这里使用Mutex和MutexGuard，主要是利用rust的Deref和Drop trait的特性（仿照了spin里面的Mutex）
 */

pub struct Mutex<T: ?Sized> {
    lock: sync::Lock,
    data: UnsafeCell<T>,
}
impl <T> Mutex<T> {
    pub const fn new(data: T) -> Self {
        Self {
            lock: sync::Lock::new(),
            data: UnsafeCell::new(data),
        }
    }

    // pub fn init(&mut self) {
    //     self.lock.init();
    // }

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



impl <T:?Sized> Deref for Mutex<T> {
    type Target = T;

    /**
     * 解引用。
     * 指定返回值的生命周期跟MutexGuard生命周期一致。
     * 这样避免了编译器的自动推断，更加准确
     */
    fn deref(&self) -> & Self::Target {
        unsafe { &*self.data.get() }
    }
}

impl <T:?Sized> DerefMut for Mutex<T> {
    fn deref_mut<>(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.data.get() }
    }
}


/**
 * 当Mutex的lock()方法得到MutexGuard，离开作用于就调用Drop trait
 */
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
        // drop的时候解锁
        self.lock.unlock();
    }
}