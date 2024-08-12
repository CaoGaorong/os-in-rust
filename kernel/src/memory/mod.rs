mod memory_allocation;
mod memory_deallocation;
mod mem_block;
mod memory_poll;
mod memory_management;
pub mod page_util;

// 初始化内存池
// 申请内存
pub use memory_management::malloc;
pub use memory_management::malloc_system;
pub use memory_management::sys_malloc;
pub use memory_management::malloc_kernel_page;
pub use memory_management::malloc_user_page_by_vaddr;

// 释放内存
pub use memory_management::sys_free;

pub use memory_management::copy_single_user_page;


pub use mem_block::MemBlockAllocator;


pub use memory_poll::get_user_mem_pool;
pub use memory_poll::mem_pool_init;
