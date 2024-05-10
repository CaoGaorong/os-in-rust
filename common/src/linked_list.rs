use core::ptr;
use crate::ASSERT;

/**
 * 定义一个链表的节点
 */
pub struct LinkedNode {
    pub next: *mut LinkedNode,
    pub pre: *mut LinkedNode,
}
impl LinkedNode {
    pub const fn new() -> Self {
        Self {
            next: ptr::null_mut(),
            pre: ptr::null_mut(),
        }
    }
}
unsafe impl Send for LinkedNode {}
unsafe impl Sync for LinkedNode {}
/**
 * 定义一个链表。有头链表。头结点和尾节点不是数据节点
 */
pub struct LinkedList {
    head: LinkedNode,
    tail: LinkedNode,
    initialized: bool,
}
impl LinkedList {

    pub const fn new() -> Self {
        Self {
            head: LinkedNode::new(),
            tail: LinkedNode::new(),
            initialized: false,
        }
    }

    pub const fn init(&mut self) {
        self.head.next = &mut self.tail;
        self.tail.pre = &mut self.head;
        self.initialized = true;
    }

    /**
     * 该链表是否为空
     */
    pub fn is_empty(&mut self) -> bool {
        if self.head.next.is_null() || self.tail.pre.is_null() {
            return true;
        }
        if self.head.next == &mut self.tail {
            return true;
        }
        return false;
    }
    /**
     * 往头部插入一个节点
     * head <-> A <-> B <-> tail。往A的前面插入一个节点
     */
    pub fn push(&mut self, node: &mut LinkedNode) {
        ASSERT!(self.initialized);
        node.next = self.head.next;
        node.pre = &mut self.head;
        (unsafe { &mut *self.head.next }).pre = node;
        self.head.next = node;
    }
    /**
     * 往尾部插入一个节点
     * head <-> A <-> B <-> tail。往B的后面插入一个节点
     */
    pub fn append(&mut self, node: &mut LinkedNode) {
        ASSERT!(self.initialized);
        node.next = &mut self.tail;
        node.pre = self.tail.pre;
        (unsafe { &mut *self.tail.pre }).next = node;
        self.tail.pre = node;
    }

    /**
     * 把第一个数据节点，弹出
     * head <-> A <-> B <-> tail。A节点弹出
     */
    pub fn pop(&mut self) -> &mut LinkedNode {
        ASSERT!(self.initialized);
        // 要弹出的节点
        let target_node = self.head.next;
        // 弹出节点右边的接口
        let right_node = unsafe { &mut *(&mut *target_node).next };
        self.head.next = right_node;
        right_node.pre = &mut self.head;
        
        unsafe { &mut *target_node }
    }
    pub fn iter(&self) -> LinkedNodeIterator {
        ASSERT!(self.initialized);
        LinkedNodeIterator {
            current: self.head.next,
            reversed: false,
        }
    }
    pub fn iter_reversed(&self) -> LinkedNodeIterator {
        ASSERT!(self.initialized);
        LinkedNodeIterator {
            current: self.tail.pre,
            reversed: true,
        }
    }

}

pub struct LinkedNodeIterator {
    current: *mut LinkedNode,
    reversed: bool,
}

impl Iterator for LinkedNodeIterator {
    type Item = *mut LinkedNode;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_null() {
            return Option::None;
        }
        let current_node = unsafe { &mut *self.current };
        // 上一个节点或者下一个节点为空，说明是head或者tail
        if current_node.next.is_null() || current_node.pre.is_null() {
            return Option::None;
        }
        if !self.reversed {
            self.current = current_node.next;
        } else {
            self.current = current_node.pre;
        }
        return Some(current_node);
    }
}