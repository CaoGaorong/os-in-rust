use core::{arch::asm, mem::size_of};

use crate::racy_cell::RacyCell;


#[no_mangle]
#[link_section = ".memeory_map"]
static MEMORY_MAP: RacyCell<[AddressRangeDescriptorStructure; 10]> = RacyCell::new([AddressRangeDescriptorStructure::empty(); 10]);



/**
 * ARDS的结构，见：
 * <https://wiki.osdev.org/Detecting_Memory_(x86)#BIOS_Function:_INT_0x15.2C_EAX_.3D_0xE820>
 * <https://uefi.org/htmlspecs/ACPI_Spec_6_4_html/15_System_Address_Map_Interfaces/Sys_Address_Map_Interfaces.html>
 */
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct AddressRangeDescriptorStructure {
    pub base_addr: u64,
    pub len: u64,
    pub region_type: u32,
}
impl AddressRangeDescriptorStructure {
    const fn empty() -> Self {
        Self {
            base_addr:0,
            len: 0,
            region_type: 0,
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


/**
 * 获取memeory_map，会得到一个List<ARDS>
 * 文档：<https://wiki.osdev.org/Detecting_Memory_(x86)#BIOS_Function:_INT_0x15.2C_EAX_.3D_0xE820>
 * 
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
 * 
 * (u32, u32) -> ARDS数组的起始地址和ARDS数组的长度
 */
#[inline]
pub fn get_memeory_map() -> (u32, u32) {
    // 最终的ARDS结构列表
    // static mut MEMORY_MAP: [AddressRangeDescriptorStrure; 100] = [AddressRangeDescriptorStrure::empty(); 100];
    let memory_map = unsafe { MEMORY_MAP.get_mut() };
    
    // 魔数
    const SMAP: u32 = 0x534D4150;
    // 一个ADRS的大小。单位字节
    const ADRS_SIZE: usize = size_of::<AddressRangeDescriptorStructure>();

    // ebx的值，表示下一个ARDS结构的位置，为0表示这是最后一个ARDS结构
    let mut next_ards = 0;


    let mut ards_cnt = 0;
    loop {
        // 从eax取出的，期待的是SMAP
        let mut exptect_smap: u32;
        // ecx，每次成功写入到ARDS的长度。为0则表示最后一个ARDS
        let mut len_written: u32;
        unsafe {
            asm!(
                "int 0x15",
                inout("eax") 0xe820 => exptect_smap,
                inout("ebx") next_ards,
                inout("ecx") ADRS_SIZE => len_written,
                in("edx") SMAP,
                in("di") &memory_map[ards_cnt]
            )
        };

        // 魔数校验
        if exptect_smap != SMAP {
            return (0, 0);
        }

        // 写入的ARDS结构为0
        if len_written == 0{
            continue;
        }

        ards_cnt += 1;
        // 最后一个ARDS结构，结束了
        if next_ards == 0 {
            break;
        }
        
    }
    // (&memory_map[0] as *const _ as u32, ards_cnt as u32)
    (memory_map.as_ptr() as u32, ards_cnt as u32)
    // Result::Ok(&mut memory_map[..ards_cnt])
}
