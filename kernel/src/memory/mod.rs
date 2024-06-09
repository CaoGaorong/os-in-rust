pub mod memory_allocation;
pub mod memory_deallocation;
pub mod mem_block;
pub mod memory_poll;

// 初始化内存池
pub use memory_poll::mem_pool_init;
// 申请内存
pub use memory_allocation::malloc;
pub use memory_allocation::malloc_kernel_page;
pub use memory_allocation::malloc_user_stack_page;

// 释放内存
pub use memory_deallocation::free;