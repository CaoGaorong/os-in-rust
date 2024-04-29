use crate::{instruction, println};


// #[track_caller]
pub fn _panic_spin(file: &str, line: u32, col: u32, condition: &str) {
    // 把中断关闭
    instruction::disable_interrupt();
    println!("!!!!!PANIC!!!!");
    println!("Panic in {} at {}:{}: {}", file, line, col, condition);
    loop {}
}

#[macro_export]
macro_rules! MY_PANIC {
    ($($arg:tt)*) => {
        $crate::assert::_panic_spin(file!(), line!(), column!(), $($arg)*);
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