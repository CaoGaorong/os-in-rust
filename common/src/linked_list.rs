use core::{ptr, sync::atomic::{AtomicBool, Ordering}};
use crate::{printkln, ASSERT};

/**
 * 定义一个链表的节点
 */
#[repr(C, packed)]
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
    cas: AtomicBool,
}
impl LinkedList {

    pub const fn new() -> Self {
        Self {
            head: LinkedNode::new(),
            tail: LinkedNode::new(),
            initialized: false,
            cas: AtomicBool::new(false)
        }
    }

    /**
     * 这个方法没法使用const修饰。
     * 并且这个方法没法放到new()里面，因为：
     *  - 当前的LinkedList成员head和tail都是结构体，赋值的时候会把栈数据按位拷贝，并且所有权转移
     *  - 而这里操作head.next和tail.pre指针，当在new()函数里面操作这个指针的话，那么指针指向的是局部变量的地址(栈内地址)
     *  - 当new()函数返回后，head和tail已经按位拷贝赋值给返回值并且转移所有权了，但是指针指向的地址已经释放了。这就是悬挂指针
     * 
     * 所以这个方法只能在程序运行期间调用，不能在编译期间调用，因此不能设定为const
     */
    fn init(&mut self) {
        // 如果已经初始化了，那就不用初始化了
        if self.initialized {
            return;
        }
        loop {
            // 如果获取锁失败，重试。自旋
            if !self.cas.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_ok() {
                continue;
            }
            // 已经初始化了，不用初始化了 double check
            if self.initialized {
                return;
            }
            // 开始初始化
            self.head.next = &mut self.tail;
            self.tail.pre = &mut self.head;
            // 设置为已经初始化
            self.initialized = true;
            // 释放乐观锁
            self.cas.store(false, Ordering::Release);
            return;
        }
    }

    /**
     * 该链表是否为空
     */
    pub fn is_empty(&mut self) -> bool {
        if self.head.next.is_null() || self.tail.pre.is_null() {
            return true;
        }
        if self.head.next as u32 == &mut self.tail as *const _ as u32 {
            return true;
        }
        return false;
    }
    /**
     * 往头部插入一个节点
     * head <-> A <-> B <-> tail。往A的前面插入一个节点
     */
    pub fn push(&mut self, node: &mut LinkedNode) {
        self.init();
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
        self.init();
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
        self.init();
        ASSERT!(self.initialized);
        // 要弹出的节点
        let target_node = self.head.next;
        // 弹出节点右边的接口
        let right_node = unsafe { &mut *(&mut *target_node).next };
        self.head.next = right_node;
        right_node.pre = &mut self.head;
        
        unsafe { &mut *target_node }
    }

    /**
     * 是否包含
     */
    pub fn contains(&mut self, node: &LinkedNode) -> bool {
        self.iter().any(|e| {
            (e as u32) == (node as *const _ as u32)
        })
    }
    pub fn iter(&self) -> LinkedNodeIterator {
        LinkedNodeIterator {
            current: self.head.next,
            reversed: false,
        }
    }
    pub fn iter_reversed(&self) -> LinkedNodeIterator {
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
