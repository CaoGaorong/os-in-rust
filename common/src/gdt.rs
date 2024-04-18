use core::{arch::asm, mem::size_of};

use crate::sd::{Granularity, GranularityEnum, SegmentDPL, SegmentDescritor, SegmentType};


/**
 * 全局描述符表
 */
#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct GlobalDecriptorTable {
    /**
     * 0号描述符
     */
    pub zero: SegmentDescritor,

    /**
     * 代码段描述符
     */
    pub code_seg: SegmentDescritor,
    /**
     * 数据段描述符
     */
    pub data_seg: SegmentDescritor,

    /**
     * 视频段描述符
     */
    pub video_seg: SegmentDescritor,
}

/**
 * 加载到GDTR的数据
 * GDTR的结构
 */
#[repr(C, packed)]
pub struct GDTR {
    /**
     * 全局描述符表的大小。表中元素的个数
     */
    gdt_limit: u16,

    /**
     * 指向全局描述符表的起始地址
     */
    gdt_ptr: u32,
}

impl GlobalDecriptorTable {
    pub const fn new() -> Self {
        let base_addr = 0x0;
        let seg_limit = 0xfffff;
        let granularity = Granularity::new(GranularityEnum::Unit4KB);
        // 代码段
        let code_segment = SegmentDescritor::new(
            base_addr,
            seg_limit,
            granularity,
            SegmentDPL::LEVEL0,
            true,
            SegmentType::NormalCodeSegment,
            false,
            false,
            true,
        );

        // 数据段
        let data_segment = SegmentDescritor::new(
            base_addr,
            seg_limit,
            granularity,
            SegmentDPL::LEVEL0,
            true,
            SegmentType::NormalDataSegment,
            false,
            false,
            true,
        );

        // 数据段
        let video_segment = SegmentDescritor::new(
            0xb8000,
            0x7,
            granularity,
            SegmentDPL::LEVEL0,
            true,
            SegmentType::NormalDataSegment,
            false,
            false,
            true,
        );

        

        // 第0个描述符，全部都是0
        let zero_val: u64 = 0;
        let zero_seg = unsafe { *(&zero_val as *const u64 as *const SegmentDescritor) };

        Self {
            zero: zero_seg,
            code_seg: code_segment,
            data_seg: data_segment,
            video_seg: video_segment
        }
    }
    pub fn compose_gdtr(&'static self) -> GDTR {
        GDTR{
            gdt_limit: (size_of::<GlobalDecriptorTable>()) as u16,
            gdt_ptr: self as *const GlobalDecriptorTable as u32,
        }
    }
}

pub fn load_gdtr(gdtr_addr: &GDTR) {
    // 加载到GDTR寄存器c
    unsafe {
        asm!("cli", "lgdt [{}]", in(reg) gdtr_addr, options(readonly, nostack, preserves_flags));
    }
    set_protected_mode_bit();
    // load GDT
    unsafe {
        asm!("mov {0}, 0x10", "mov ds, {0}", "mov ss, {0}", out(reg) _);
    }

}
fn set_protected_mode_bit() -> u32 {
    let mut cr0: u32;
    unsafe {
        asm!("mov {:e}, cr0", out(reg) cr0, options(nomem, nostack, preserves_flags));
    }
    let cr0_protected = cr0 | 1;
    write_cr0(cr0_protected);
    cr0
}

fn write_cr0(val: u32) {
    unsafe { asm!("mov cr0, {:e}", in(reg) val, options(nostack, preserves_flags)) };
}
