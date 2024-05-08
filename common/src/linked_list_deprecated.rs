use core::ptr;

trait ListNodeTrait: Copy{}
/**
 * 构建一个链表。
 * 不能使用堆内存
 */

/**
 * 链表节点。这里只能使用裸指针，不能使用可变借用。因为所有权问题，又不能使用Rc，所以没法使用借用
 */
pub struct LinkedNode<T: ListNodeTrait> {
    /**
     * 当前节点的值
     */
    value: T,
    /**
     * 下一个节点的指针
     */
    next: *mut LinkedNode<T>,
    /**
     * 上一个节点的指针
     */
    pre: *mut LinkedNode<T>,
}

impl<T: ListNodeTrait> LinkedNode<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: value,
            next: ptr::null_mut(),
            pre: ptr::null_mut(),
        }
    }
}

/**
 * 构建一个无头链表（第一个节点就是数据节点）
 */
pub struct LinkedList<T: ListNodeTrait> {
    /**
     * 链表的头
     */
    head: *mut LinkedNode<T>,
    /**
     * 链表的尾部
     */
    tail: *mut LinkedNode<T>,
}
impl<T: ListNodeTrait> LinkedList<T> {
    /**
     * 构建一个空的双向链表
     */
    pub const fn new() -> Self {
        Self {
            head: ptr::null_mut(),
            tail: ptr::null_mut(),
        }
    }
    /**
     * 是否空链表
     */
    pub fn is_empty(&self) -> bool {
        self.head.is_null() && self.tail.is_null()
    }
    /**
     * 往链表头插入一个节点
     * A <-> B，往A前面插入node
     */
    pub fn push(&mut self, node: &mut LinkedNode<T>) {
        if self.is_empty() {
            self.head = node;
            self.tail = node;
            return;
        }
        // 新节点，指向旧head
        node.next = self.head;
        // 旧head指向新节点
        (unsafe { &mut *self.head }).pre = node;
        // 该节点作为新head
        self.head = node;
    }
    /**
     * 往链表尾部插入一个节点
     * A <-> B，往B后面插入node
     */
    pub fn append(&mut self, node: &mut LinkedNode<T>) {
        if self.is_empty() {
            self.head = node;
            self.tail = node;
            return;
        }
        // 新节点，指向旧tail
        node.pre = self.tail;
        // 旧tail下一级指向新接口
        (unsafe { &mut *self.tail }).next = node;
        // 新节点，作为新tail
        self.tail = node;
    }

    /**
     * 弹出第一个节点
     * A <-> B <-> C，弹出节点A
     */
    pub fn pop(&mut self) -> &mut LinkedNode<T> {
        // 要弹出的节点A
        let target_node = self.head;
        // 节点B变成头节点了
        self.head = (unsafe { &*self.head }).next;
        unsafe { &mut *target_node }
    }

    /**
     * 得到一个迭代器
     */
    pub fn iter(&self) -> LinkedNodeIterator<T> {
        LinkedNodeIterator { 
            current: self.head,
            reversed: false,
        }
    }
    /**
     * 得到一个逆向遍历的迭代器
     */
    pub fn iter_reversed(&self) -> LinkedNodeIterator<T> {
        LinkedNodeIterator { 
            current: self.tail,
            reversed: true,
        }
    }

}

pub struct LinkedNodeIterator<T: ListNodeTrait> {
    current: *const LinkedNode<T>,
    reversed: bool,
}

impl<T: ListNodeTrait> Iterator for LinkedNodeIterator<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_null() {
            return Option::None;
        }
        let current_node = unsafe { &*self.current };
        let value = current_node.value;
        if self.reversed {
            self.current = current_node.pre;
        } else {
            self.current = current_node.next;
        }
        Option::Some(value)
    }
}
