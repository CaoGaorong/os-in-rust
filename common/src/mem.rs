use core::{arch::asm, mem::{self, size_of}};

use crate::{bitmap::MemoryError, println, racy_cell::RacyCell};

// #[no_mangle]
static MEMORY_MAP: RacyCell<[AddressRangeDescriptorStrure; 50]> = RacyCell::new([AddressRangeDescriptorStrure::empty(); 50]);
/**
 * 获取memeory_map，会得到一个List<ARDS>
 * 入参：
 *      - eax: 功能码，为0xe820
 *      - ebx: 下一个ARDS结构的地址，首次调用传入0
 *      - es:di: 执行中断后，得到的ARDS会写入的地址
 *      - ecx: ARDS结构的大小，固定为20字节
 *      - edx: 魔数0x534D4150
 * 返回值：
 *      - CF: 发生错误则CF = 1，否则为0
 *      - eax: 返回魔数0x534D4150
 *      - ebx: ARDS所在的下一个区域，为0则表示ARDS是最后一个
 *      - es:di: 跟入参一致。如果要把每次得到的ARDS保存，那么需要自己手动调节es:di指向的地址
 *      - ecx: ARDS的大小
 */
#[no_mangle]
pub fn get_memeory_map1() -> Result<&'static [AddressRangeDescriptorStrure], ()> {
    // 最终的ARDS结构列表
    static mut MEMORY_MAP: [AddressRangeDescriptorStrure; 100] = [AddressRangeDescriptorStrure::empty(); 100];
    
    // 魔数
    const SMAP: u32 = 0x534D4150;

    // ebx的值，表示下一个ARDS结构的位置，为0表示这是最后一个ARDS结构
    let mut next_ards = 0;

    // 调用中断后，得到的ARDS保存的地方，缓冲区
    let buf = [0u8; 20];

    let mut ards_cnt = 0;
    loop {
        // 从eax取出的，期待的是SMAP
        let exptect_smap: u32;
        // ecx，每次成功写入到ARDS的长度。为0则表示最后一个ARDS
        let len_written;
        unsafe {
            asm!(
                "push ebx",
                "mov ebx, edx",
                "mov edx, 0x534D4150",
                "int 0x15",
                "mov edx, ebx",
                "pop ebx",
                inout("eax") 0xe820 => exptect_smap,
                inout("edx") next_ards,
                inout("ecx") buf.len() => len_written,
                in("di") &buf
            )
        };

        if exptect_smap != SMAP {
            return Err(());
        }

        // 写入的ARDS结构为0
        if len_written == 0{
            continue;
        }

        let buf = &buf[..len_written];
        // 从缓冲区中得到ARDS
        let ards = unsafe { *(buf as *const [u8] as *const AddressRangeDescriptorStrure) };

        unsafe {
            MEMORY_MAP[ards_cnt] = ards;
        }
        ards_cnt += 1;

        // 最后一个ARDS结构，结束了
        if next_ards == 0 {
            break;
        }
    }
    unsafe {
        Result::Ok(&MEMORY_MAP[..ards_cnt])
    }
}

pub unsafe fn query_memory_map() -> (u32, u32) {
    
    const SMAP: u32 = 0x534D4150;

    let memory_map = unsafe { MEMORY_MAP.get_mut() };

    let mut i = 0;

    // let mut offset = 0;
    // let buf = [0u8; 20];
    // loop {
    //     let ret: u32;
    //     let buf_written_len;
    //     unsafe {
    //         asm!(
    //             "push ebx",
    //             "mov ebx, edx",
    //             "mov edx, 0x534D4150",
    //             "int 0x15",
    //             "mov edx, ebx",
    //             "pop ebx",
    //             inout("eax") 0xe820 => ret,
    //             inout("edx") offset,
    //             inout("ecx") buf.len() => buf_written_len,
    //             in("di") &buf
    //         )
    //     };
    //     if ret != SMAP {
    //         return (0, 0);
    //     }

    //     if buf_written_len != 0 {
    //         let buf = &buf[..buf_written_len];

    //         let (&base_raw, rest) = split_array_ref(buf);
    //         let (&len_raw, rest) = split_array_ref(rest);
    //         let (&kind_raw, rest) = split_array_ref(rest);
    //         let acpi_extended_raw: [u8; 4] = rest.try_into().unwrap_or_default();

    //         let len = u64::from_ne_bytes(len_raw);
    //         if len != 0 {
    //             memory_map[i] = AddressRangeDescriptorStrure {
    //                 base_addr: u64::from_ne_bytes(base_raw),
    //                 len,
    //                 region_type: u32::from_ne_bytes(kind_raw),
    //             };
    //             i += 1;
    //         }
    //     }

    //     if offset == 0 {
    //         break;
    //     }
    // }

    // Ok(&mut memory_map[..i])
    (memory_map as *const _ as u32, i as u32)
}

// #[no_mangle]
// pub unsafe fn query_memory_map2() -> u32 {
//     const SMAP: u32 = 0x534D4150;

//     let memory_map = unsafe { MEMORY_MAP.get_mut() };

//     let mut i = 0;

//     let mut offset = 0;
//     let buf = [0u8; 24];
//     loop {
//         let ret: u32;
//         let buf_written_len;
//         unsafe {
//             asm!(
//                 "push ebx",
//                 "mov ebx, edx",
//                 "mov edx, 0x534D4150",
//                 "int 0x15",
//                 "mov edx, ebx",
//                 "pop ebx",
//                 inout("eax") 0xe820 => ret,
//                 inout("edx") offset,
//                 inout("ecx") buf.len() => buf_written_len,
//                 in("di") &buf
//             )
//         };
//         if ret != SMAP {
//             return 0
//         }

//         if buf_written_len != 0 {
//             let buf = &buf[..buf_written_len];

//             let (&base_raw, rest) = split_array_ref(buf);
//             let (&len_raw, rest) = split_array_ref(rest);
//             let (&kind_raw, rest) = split_array_ref(rest);
//             let acpi_extended_raw: [u8; 4] = rest.try_into().unwrap_or_default();

//             let len = u64::from_ne_bytes(len_raw);
//             if len != 0 {
//                 memory_map[i] = AddressRangeDescriptorStrure {
//                     base_addr: u64::from_ne_bytes(base_raw),
//                     len,
//                     region_type: u32::from_ne_bytes(kind_raw),
//                     acpi_extended_attributes: u32::from_ne_bytes(acpi_extended_raw),
//                 };
//                 i += 1;
//             }
//         }

//         if offset == 0 {
//             break;
//         }
//     }

//     // Ok((memory_map.as_ptr() as u32, i as u32))
//     memory_map.as_ptr() as u32
// }

fn split_array_ref<const N: usize, T>(slice: &[T]) -> (&[T; N], &[T]) {
    if N > slice.len() {
        fail(b'S');
    }
    let (a, b) = slice.split_at(N);
    // SAFETY: a points to [T; N]? Yes it's [T] of length N (checked by split_at)
    unsafe { (&*(a.as_ptr() as *const [T; N]), b) }
}


#[cold]
#[inline(never)]
#[no_mangle]
pub extern "C" fn fail(code: u8) -> ! {
    panic!("fail: {}", code as char);
}

/**
 * ARDS的结构，见：
 * <https://wiki.osdev.org/Detecting_Memory_(x86)#BIOS_Function:_INT_0x15.2C_EAX_.3D_0xE820>
 */
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct AddressRangeDescriptorStrure {
    base_addr: u64,
    len: u64,
    region_type: u32,
    // 还有一个Extended Attributes bitfield，不要了
    // acpi_extended_attributes: u32
}
impl AddressRangeDescriptorStrure {
    const fn empty() -> Self {
        Self {
            base_addr:0,
            len: 0,
            region_type: 0,
            // acpi_extended_attributes: 0,
        }
    }
}

/**
 * ARDS region的类型：
 * Next uint32_t = Region "type"
 *   Type 1: Usable (normal) RAM
 *   Type 2: Reserved - unusable
 *   Type 3: ACPI reclaimable memory
 *   Type 4: ACPI NVS memory
 *   Type 5: Area containing bad memory
 */
pub enum ARDSType {
    Usable = 0x1,
    Reserved,
    AcpiReclaimable,
    AcpiNvs,
    AreaBad,
}


