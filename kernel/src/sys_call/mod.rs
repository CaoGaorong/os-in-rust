mod sys_call;
mod sys_call_api;
mod sys_call_proxy;
mod dir_api;
mod file_api;

pub use sys_call_api::init;
pub use sys_call::HandlerType;
pub use sys_call::get_handler;

pub use sys_call_proxy::get_pid;
pub use sys_call_proxy::write;
pub use sys_call_proxy::malloc;
pub use sys_call_proxy::free;
pub use sys_call_proxy::read_key;
pub use sys_call_proxy::fork;
pub use sys_call_proxy::ForkResult;
pub use sys_call_proxy::thread_yield;
pub use sys_call_proxy::clear_screen;
pub use sys_call_proxy::sys_print;
pub use sys_call_proxy::exec;
pub use crate::println;
pub use crate::print;


pub use dir_api::create_dir;
pub use dir_api::create_dir_all;
pub use dir_api::read_dir;
pub use dir_api::remove_dir;


pub use file_api::File;
