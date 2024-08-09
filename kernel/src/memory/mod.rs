pub mod memory_allocation;
pub mod memory_deallocation;
pub mod mem_block;
pub mod memory_poll;
mod memory_management;

// 初始化内存池
// 申请内存
pub use memory_management::malloc;
pub use memory_management::malloc_system;
pub use memory_management::sys_malloc;
pub use memory_management::malloc_kernel_page;
pub use memory_management::malloc_user_page_by_vaddr;

// 释放内存
pub use memory_management::sys_free;

pub use memory_management::copy_single_page;