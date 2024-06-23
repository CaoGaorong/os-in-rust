use core::fmt;

use crate::{instruction, printkln, vga};


// #[track_caller]
pub fn _panic_spin(file: &str, line: u32, col: u32, condition: fmt::Arguments) {
    // 把中断关闭
    instruction::disable_interrupt();
    printkln!("!!!!!PANIC!!!!");
    printkln!("Panic in {} at {}:{}", file, line, col);
    vga::print(condition);
    printkln!();

    loop {}
}

#[macro_export]
macro_rules! MY_PANIC {
    ($($arg:tt)*) => {
        $crate::assert::_panic_spin(file!(), line!(), column!(), format_args!($($arg)*));
    };
}

// 非debug，不用判断
// #[cfg(not(debug_assertions))]
// macro_rules! MY_PANIC {
//     ($($arg:tt)*) => {
//         ();
//     };
// }


#[macro_export]
// #[cfg(debug_assertions)]
macro_rules! ASSERT {
    ($condition:expr) => {
        if $condition {}
        else {
            $crate::MY_PANIC!(stringify!($condition));
        }
    };
}

// 非debug，不用判断
// #[cfg(not(debug_assertions))]
// macro_rules! ASSERT {
//     ($condition:expr) => {
//         ();
//     };
// }