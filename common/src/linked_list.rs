use core::{ptr, sync::atomic::{AtomicBool, Ordering}};
use crate::{printkln, ASSERT, MY_PANIC, printk};

/**
 * 定义一个链表的节点
 */
#[derive(Debug)]
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
    pub const fn init(&mut self) {
        self.pre = ptr::null_mut();
        self.next = ptr::null_mut();
    }
}
unsafe impl Send for LinkedNode {}
unsafe impl Sync for LinkedNode {}
/**
 * 定义一个链表。无头链表
 */
#[derive(Debug)]
pub struct LinkedList {
    head: *mut LinkedNode,
    tail: *mut LinkedNode,
    lock: AtomicBool,
}

// 自己保证并发问题
unsafe impl Send for LinkedList {}
unsafe impl Sync for LinkedList {}

impl LinkedList {

    pub const fn new() -> Self {
        Self {
            head: ptr::null_mut(),
            tail: ptr::null_mut(),
            lock: AtomicBool::new(false)
        }
    }

    /**
     * 该链表是否为空
     */
    pub fn is_empty(&self) -> bool {
        if self.head == ptr::null_mut() || self.tail == ptr::null_mut() {
            return true;
        }
        return false;
    }

    /**
     * 自旋锁
     */
    fn lock(&self) {
        loop {
            // 获取到锁了
            if self.lock.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_ok() {
                return
            }
        }
    }
    fn unlock(&self) {
        self.lock.store(false, Ordering::Release);
    }
    /**
     * 往头部插入一个节点
     * A <-> B。往A的前面插入一个节点
     * |     |
     * head tail
     */
    pub fn push(&mut self, node: &mut LinkedNode) {
        // 初始化，清空数据
        node.init();

        self.lock();
        // 如果链表为空，
        if self.is_empty() {
            // 链表的头和尾都是该节点
            self.head = node;
            self.tail = node;
            self.unlock();
            return;
        }

        // 原先的头结点
        let first_node = unsafe { &mut *self.head };

        // node -> exist_node
        node.next = first_node;
        // node <- exist_node
        first_node.pre = node;

        // 设置为头结点
        self.head = node;

        self.unlock();
    }
    /**
     * 往尾部插入一个节点
     * A <-> B。往B的后面插入一个节点
     * |     |
     * head tail
     */
    pub fn append(&mut self, node: &mut LinkedNode) {
        // 初始化，清空数据
        node.init();

        self.lock();

        if self.is_empty() {
            // 链表的头和尾都是该节点
            self.head = node;
            self.tail = node;
            self.unlock();
            return;
        }
        // 原先排最后的节点
        let last_node =  unsafe { &mut *self.tail };

        // 现在这个节点最后了
        node.pre = last_node;
        last_node.next = node;

        self.tail = node;

        self.unlock();
    }

    /**
     * 把第一个数据节点，弹出
     * head <-> A <-> B <-> tail。A节点弹出
     */
    pub fn pop(&mut self) -> &mut LinkedNode {
        self.lock();
        ASSERT!(!self.is_empty());
        // 要弹出的节点
        let target_node = unsafe { &mut *self.head };
        // 下一个节点变成了头节点
        self.head = target_node.next;

        self.refresh();
        self.unlock();
        target_node.pre = ptr::null_mut();
        target_node.next = ptr::null_mut();
        target_node
    }

    /**
     * 移除链表中的某一个节点
     * 比如head <-> A <-> B <-> C <-> tail，要移除B
     *     pre: A节点；next: C节点
     *     head <-> A <-> C <-> tail
     */
    pub fn remove(&mut self, node: &LinkedNode) {
        self.lock();
        if !self.contains(node) {
            self.unlock();
            return;
        }
        // 该节点的上一个节点
        let pre = unsafe { &mut *node.pre };
        // 该节点的下一个节点
        let next = unsafe { &mut *node.next };

        if pre.next != ptr::null_mut() {
            pre.next = next as *mut _;
        }
        if next.pre != ptr::null_mut() {
            next.pre = pre as *mut _;
        }
        self.refresh();
        self.unlock();
    }

    /**
     * 是否包含
     */
    pub fn contains(&mut self, node: &LinkedNode) -> bool {
        self.iter().any(|e| {
            (e as u32) == (node as *const _ as u32)
        })
    }

    /**
     * 刷新一下。
     * 因为删除元素的时候，头尾节点一直在变，但是为了保证遍历的时候的正确性，所以需要每次置空元素
     * 因为在遍历元素的时候，是通过是否为null，从而判断是否结束的
     */
    fn refresh(&mut self) {
        // 头结点没有上级
        unsafe { &mut *self.head}.pre = ptr::null_mut();
        // 尾结点没有上级
        unsafe { &mut *self.tail}.next = ptr::null_mut();
    }

    pub fn size(&self) -> usize {
        self.iter().count()
    }
    pub fn iter(&self) -> LinkedNodeIterator {
        LinkedNodeIterator {
            current: self.head,
            reversed: false,
        }
    }
    pub fn iter_reversed(&self) -> LinkedNodeIterator {
        LinkedNodeIterator {
            current: self.tail,
            reversed: true,
        }
    }

    pub fn print_list(&self) {
        printkln!("print list:");
        self.iter().for_each(|node| {
            if node  == ptr::null_mut() {
                return;
            }
            let cur_node = unsafe { &*node };
            printk!("0x{:x}: {:?} ", node as usize, cur_node)
        });
        printkln!();
    }

}

pub struct LinkedNodeIterator {
    current: *mut LinkedNode,
    reversed: bool,
}

impl Iterator for LinkedNodeIterator {
    type Item = *mut LinkedNode;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current == ptr::null_mut() {
            return Option::None;
        }
        let current_node = unsafe { &mut *self.current };
        if self.reversed {
            self.current = current_node.pre;
        } else {
            self.current = current_node.next;
        }
        return Some(current_node);
    }
}
