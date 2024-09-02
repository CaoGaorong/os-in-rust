use core::ops::DerefMut;

use os_in_rust_common::{constants, paging::PageTable, pool::MemPool, printk};

use crate::{filesystem::{FileDescriptor, FileDescriptorType, StdFileDescriptor}, memory, pid_allocator::Pid, pipe, scheduler, thread::{self, TaskStatus, TaskStruct}};

pub type TaskExitStatus = u8;

/**
 * exit系统调用。当某个用户进程调用exit，那么就需要释放这个用户进程的空间
 *   当前任务的所有资源都释放，只剩下当前任务的PCB还在
 */
#[inline(never)]
pub fn exit(status: TaskExitStatus) {
    let cur_task = &mut thread::current_thread().task_struct;
    // 如果已经退出过了，那么不再处理
    if cur_task.task_status == TaskStatus::TaskHanging {
        return;
    }
    cur_task.exit_status = Option::Some(status);
    
    cur_task.check_stack_magic("failed to exit");

    // 把管道关掉
    self::close_pipe(cur_task);

    // 自己要退出了，把子进程过继给init
    self::trans_children_to_init(cur_task);
    cur_task.check_stack_magic("failed to trans children to init");

    // 把当前任务的堆内存资源给释放掉（根据虚拟地址，找到内存空间）
    self::release_heap_resource(&cur_task.vaddr_pool);
    cur_task.check_stack_magic("failed to release heap resource");
    
    // 把当前任务的虚拟内存池自身给释放掉
    self::release_vaddr_pool(&cur_task.vaddr_pool);
    cur_task.check_stack_magic("failed to release vaddr pool");

    // 释放这个任务的页表
    self::release_dir_table(unsafe { &*cur_task.pgdir });
    cur_task.check_stack_magic("failed to release page dir table");

    // 把当前任务的父任务，给唤醒（如果在等待的话）
    self::wake_up_parent(cur_task);


    // 当前进程退出了，设置为挂着状态
    scheduler::block_thread(cur_task, TaskStatus::TaskHanging);
}

/**
 * 把cur_task任务的子进程过继给init进程
 */
#[inline(never)]
fn trans_children_to_init(task: &TaskStruct) {
    let child = task.find_child();
    if child.is_none() {
        return;
    }
    // 把这个任务的父，设定为init进程
    // TODO 怎么确保init进程的pid就是1？
    child.unwrap().parent_pid = Option::Some(Pid::new(1));
}

/**
 * 释放当前进程的堆内存资源（用户空间，低端3GB）
 */
#[inline(never)]
fn release_heap_resource(vaddr_pool: &MemPool) {
    // 把虚拟地址指向的物理空间都给删掉
    memory::free_by_addr_pool(vaddr_pool);
}

/**
 * 释放当前进程的虚拟内存池资源
 */
#[inline(never)]
fn release_vaddr_pool(vaddr_pool: &MemPool) {
    let bitmap = vaddr_pool.bitmap.get_bitmap();
    memory::free_kernel_page(bitmap.as_ptr() as usize, bitmap.len() / constants::PAGE_SIZE as usize, true);
}

/**
 * 释放页目录表自身的空间
 */
#[inline(never)]
fn release_dir_table(dir_table: &PageTable) {
    // 把页表所在的地址，这一页给释放掉
    memory::free_kernel_page(dir_table as *const _ as usize, 1, true);
}


/**
 * 把任务的父任务给唤醒（如果在等待子任务的话）
 */
#[inline(never)]
fn wake_up_parent(cur_task: &TaskStruct) {
    // 找到当前任务的父任务
    let parent_task = cur_task.find_parent();
    if parent_task.is_some() {
        let parent_task = parent_task.unwrap();
        // 如果父任务在等待子任务，那么就把父任务给唤醒
        if parent_task.task_status == TaskStatus::TaskWaiting {
            thread::wake_thread(parent_task);
        }
    }
}

/**
 * 任务结束，pipe写入完毕
 */
#[inline(never)]
fn close_pipe(cur_task: &TaskStruct) {
    let pipe_list = pipe::get_pipe_list();
    for pipe_container in pipe_list {
        if pipe_container.is_none() {
            continue;
        }
        let pipe = pipe_container.as_mut().unwrap();
        
        // 生产者退出，那么往管道里写入结束
        if pipe.get_producer() as *const _ ==  cur_task as *const _ {
            pipe.write_end();
        }

        // 消费者退出，那么管道销毁
        if pipe.get_consumer() as *const _ == cur_task as *const _ {
            *pipe_container = Option::None;
        }
    }
}