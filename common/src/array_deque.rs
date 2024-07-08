use crate::ASSERT;

#[derive(Copy, Clone)]
pub struct ArrayDeque<T:Copy> {
    ptr: *mut T,
    len: usize,
    cap: usize
}

impl <T:Copy>ArrayDeque<T> {
    pub fn new(array: &mut [T]) -> Self {
        Self {
            ptr: array.as_mut_ptr(),
            len: 0,
            cap: array.len(),
        }
    }

    pub fn get_array(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts_mut(self.ptr, self.cap)}
    }

    pub fn get_mut_array(&mut self) -> &mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.ptr, self.cap)}
    }

    pub fn append(&mut self, data: T) {
        let len = self.len;
        ASSERT!(len < self.cap);
        let arr = self.get_mut_array();
        arr[len] = data;
        self.len += 1;
    }

    pub fn push(&mut self, data: T) {
        let len = self.len;
        ASSERT!(len < self.cap);
        let arr = self.get_mut_array();
        arr.copy_within(0..len, 1);
        arr[0] = data;
        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len <= 0 {
            return Option::None;
        }
        let arr = self.get_mut_array();
        let target = arr[0];
        arr.copy_within(1.., 0);
        self.len -= 1;
        return Option::Some(target);
    }

    pub fn iter(&self) -> ArrayDequeIterator<T> {
        ArrayDequeIterator {
            deque: self,
            idx: 0,
        }
    }

}

pub struct ArrayDequeIterator<T:Copy> {
    deque: *const ArrayDeque<T>,
    idx: usize,
}

impl <T:Copy> Iterator for ArrayDequeIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let deque = &unsafe { *self.deque };
        if self.idx >= deque.len {
            return Option::None;
        }
        let target = deque.get_array()[self.idx];
        self.idx += 1;
        Option::Some(target)
    }
}

