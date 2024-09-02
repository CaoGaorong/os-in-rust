
#[derive(Copy, Clone)]
pub struct ArrayDeque<T: Copy, const N: usize> {
    data: [T; N],
    len: usize,
}

impl<T: Copy, const N: usize> ArrayDeque<T, N> {
    pub const fn new(array: [T; N]) -> Self {
        Self {
            data: array,
            len: 0,
        }
    }

    #[inline(never)]
    pub fn get_array(&self) -> &[T] {
        &self.data[..self.len]
    }

    #[inline(never)]
    pub fn append(&mut self, data: T) {
        let len = self.len;
        if len >= self.cap() {
            panic!("exceed deque capacity");
        }
        let arr = &mut self.data;
        arr[len] = data;
        self.len += 1;
    }

    #[inline(never)]
    pub fn push(&mut self, data: T) {
        let len = self.len;
        assert!(len < self.cap());
        let arr = &mut self.data;
        arr.copy_within(0..len, 1);
        arr[0] = data;
        self.len += 1;
    }

    #[inline(never)]
    pub fn pop(&mut self) -> Option<T> {
        if self.len <= 0 {
            return Option::None;
        }
        let arr = &mut self.data;
        let target = arr[0];
        arr.copy_within(1.., 0);
        self.len -= 1;
        return Option::Some(target);
    }

    #[inline(never)]
    pub fn pop_last(&mut self) -> Option<T> {
        let len = self.len;
        if len <= 0 {
            return Option::None;
        }
        let arr = &mut self.data;
        let target = arr[len - 1];
        self.len -= 1;
        return Option::Some(target);
    }


    #[inline(never)]
    pub fn size(&self) -> usize {
        self.len
    }


    #[inline(never)]
    pub fn cap(&self) -> usize {
        self.data.len()
    }


    /**
     * 清除这个队列
     */
    #[inline(never)]
    pub fn clear(&mut self) {
        self.len = 0;
    }

    #[inline(never)]
    pub fn iter(&self) -> ArrayDequeIterator<T, N> {
        ArrayDequeIterator {
            deque: self,
            idx: 0,
        }
    }
}

pub struct ArrayDequeIterator<'a, T: Copy, const N: usize> {
    deque: &'a ArrayDeque<T, N>,
    idx: usize,
}

impl<'a, T: Copy, const N: usize> Iterator for ArrayDequeIterator<'a, T, N> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let deque = self.deque;
        if self.idx >= deque.size() {
            return Option::None;
        }
        let target = deque.data[self.idx];
        self.idx += 1;
        Option::Some(target)
    }
}
