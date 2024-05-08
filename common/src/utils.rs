pub const fn bool_to_int(b: bool) -> u32 {
    if b {
        1
    } else {
        0
    }
}

pub const fn bool_to_u8 (b: bool) -> u8 {
    if b {
        1
    } else {
        0
    }
}

/**
 * 计算某个结构体的成员变量所在结构体的偏移量。比如：
 * struct MyStruct  {
 *     id: u8,
 *     age: u32,
 *     sex: u8
 * }
 * fn main() {
 *     let offset = offset!(MyStruct, sex);
 *     println("offset:{}", offset);
 * }
 * 
 * 注意rust的字节默认对齐
 */
#[macro_export]
macro_rules! offset {
    ($struct_type:ty, $member:ident) => {
        unsafe { 
            &(*(0 as *const $struct_type)).$member as *const _ as usize
        }
    };
}

/**
 * 已知某个结构体和成员以及该成员的偏移量，得到该结构体的指针
 * fn main() {
 * 
 *   let my_struct = MyStruct {
 *       id: 1,
 *       age: 20,
 *       sex: 1,
 *   };
 * 
 *   let s = elem2entry!(MyStruct, age, &my_struct.age as *const u32 as usize);
 *   let generated_struct = unsafe { &mut *s };
 * 
 *   assert_eq!(&my_struct as *const _ as u32, generated_struct as *const _ as u32);
 * }
 * 
 */
#[macro_export]
macro_rules! elem2entry {
    ($struct_type:ty, $struct_member_name:ident, $elem_ptr:expr) => {
        {
            let offset = $crate::offset!($struct_type, $struct_member_name);
            ($elem_ptr as usize - offset) as *mut $struct_type
        }
    };
}
