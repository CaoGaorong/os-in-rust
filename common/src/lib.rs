#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(const_mut_refs)]

pub mod dap;
pub mod disk;
pub mod vga;
pub mod sd;
pub mod gdt;
pub mod reg_cr0;
pub mod reg_cr3;
pub mod selector;
pub mod instruction;
pub mod utils;
pub mod racy_cell;
pub mod paging;
pub mod constants;
pub mod idt;
pub mod port;
pub mod pic;
pub mod assert;
pub mod bitmap;
pub mod pool;
pub mod context;
pub mod bios_mem;
pub mod linked_list_deprecated;
pub mod linked_list;
pub mod pit;
pub mod reg_eflags;
pub mod queue;
pub mod cstring_utils;
pub mod domain;