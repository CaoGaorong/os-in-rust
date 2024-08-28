/**
 * 队列的实现
 */

pub trait Queue<T: Copy> {
    /**
     * 往队尾追加数据
     */
    fn append(&mut self, data: T) -> Result<bool, QueueError>;
    /**
     * 取出数据
     */
    fn poll(&mut self) -> Option<T>;
    /**
     * 队列大小
     */
    fn size(&self) -> u32;

    /**
     * 是否队列的数据空了
     */
    fn is_empty(&self) -> bool;

    /**
     * 是否队列的数据满了
     */
    fn is_full(&self) -> bool;
}

/**
 * 数组实现的队列。不可动态扩容
 */
pub struct ArrayQueue<'a, T: 'a + Copy> {
    buffer: &'a mut [T],
    len: usize, 
}
impl <'a, T: Copy> ArrayQueue<'a, T> {
    pub const fn new(data: &'a mut [T]) -> Self {
        Self { 
            buffer: data,
            len: 0, 
        }
    }

    pub fn get_data(&mut self) -> &mut[T] {
        self.buffer
    }
}

pub enum QueueError {
    QueueElementExceed,
}

impl<'a, T: Copy> Queue<T> for ArrayQueue<'a, T> {
    /**
     * 追加元素
     */
    fn append(&mut self, data: T) -> Result<bool, QueueError> {
        if self.is_full() {
            return Result::Err(QueueError::QueueElementExceed);
        }
        let idx = self.len;
        self.buffer[idx] = data;
        self.len += 1;
        Result::Ok(true)
    }

    /**
     * 从队列头取出一个元素出来
     */
    fn poll(&mut self) -> Option<T> {
        if self.size() == 0 {
            return Option::None;
        }
        let e = self.buffer[0];
        // 把数组元素，往前挪动
        for i in 0..self.buffer.len() - 1 {
            self.buffer[i] = self.buffer[i + 1];
        }
        self.len -= 1;
        Option::Some(e)
    }

    fn size(&self) -> u32 {
        self.len as u32
    }
    
    fn is_empty(&self) -> bool {
        self.len == 0
    }


    fn is_full(&self) -> bool {
        self.len == self.buffer.len()
    }
}
