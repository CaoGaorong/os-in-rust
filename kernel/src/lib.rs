#![cfg_attr(test, allow(unused_imports))]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![feature(abi_x86_interrupt)]
#![feature(const_mut_refs)]
#![feature(naked_functions)]
#![feature(panic_info_message)]

pub mod interrupt;
pub mod init;
pub mod thread_management;
pub mod scheduler;
pub mod sync;
pub mod mutex;
pub mod console;
pub mod keyboard;
pub mod scancode;
pub mod printer;
pub mod blocking_queue;
pub mod tss;
pub mod memory;
pub mod process;
pub mod thread;
pub mod sys_call;
pub mod pid_allocator;
pub mod page_util;
pub mod device;
pub mod filesystem;