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
    fn poll(&mut self) -> T;
    /**
     * 队列大小
     */
    fn size(&self) -> u32;
}

/**
 * 数组实现的队列。不可动态扩容
 */
pub struct ArrayQueue<'a, T: 'a + Copy> {
    buffer: &'a mut [T],
    len: usize, 
}
impl <'a, T: Copy> ArrayQueue<'a, T> {
    pub fn new(data: &'a mut [T]) -> Self {
        Self { 
            buffer: data,
            len: 0, 
        }
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
        if self.len == self.buffer.len() {
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
    fn poll(&mut self) -> T {
        let e = self.buffer[0];
        // 把数组元素，往前挪动
        for i in 0..self.buffer.len() - 1 {
            self.buffer[i] = self.buffer[i + 1];
        }
        self.len -= 1;
        e
    }

    fn size(&self) -> u32 {
        self.len as u32
    }
}

