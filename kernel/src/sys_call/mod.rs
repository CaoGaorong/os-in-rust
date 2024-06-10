pub mod sys_call;
pub mod sys_call_api;
pub mod sys_call_proxy;

pub use sys_call_proxy::get_pid;
pub use sys_call_proxy::write;
pub use sys_call_proxy::malloc;
pub use sys_call_proxy::free;