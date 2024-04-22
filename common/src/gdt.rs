use core::{arch::asm, mem::size_of, ptr::addr_of};

use crate::sd::{Granularity, GranularityEnum, SegmentDPL, SegmentDescritor, SegmentType};

#[no_mangle]
static mut GDT: GlobalDecriptorTable = GlobalDecriptorTable::new();

/**
 * 加载GDTR
 */
pub fn load_gdt() {
    // 加载GDT到GDTR寄存器中
    unsafe { GDT.load_gdtr() };
}

/**
 * 获取GDT的地址
 */
pub fn get_gdt_addr() -> *const GlobalDecriptorTable {
    unsafe { addr_of!(GDT) }
}

/**
 * 指定GDT的地址，加载到GDTR
 */
pub fn load_gdtr_by_addr(gdt_addr: *const GlobalDecriptorTable) {
    let gdtr = GDTR::new(gdt_addr);
    // 加载到GDTR寄存器c
    unsafe {
        asm!("cli", "lgdt [{}]", in(reg) &gdtr, options(readonly, nostack, preserves_flags));
    }
}


/**
 * 填充GDT的描述符
 */
pub fn set_descriptor(desc_type: DecriptorType, descriptor: SegmentDescritor) {
    unsafe { GDT.set_descriptor(desc_type, descriptor) };
}

/**
 * 获取描述符数据（Copy过来的）
 */
pub fn get_descriptor(desc_type: DecriptorType) -> SegmentDescritor {
    unsafe { *GDT.get_descriptor(desc_type) }
}


/**
 * 全局描述符表
 */
#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct GlobalDecriptorTable {
    data: [SegmentDescritor; 4]
}

pub enum DecriptorType {
    Zero = 0x0,
    Code = 0x1,
    Data = 0x2,
    Video = 0x3
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
impl GDTR {
    /**
     * 构建GDTR的结构
     * gdt: 全局描述符表的起始地址
     */
    pub fn new(gdt: *const GlobalDecriptorTable) -> Self {
        GDTR{
            gdt_limit: (size_of::<GlobalDecriptorTable>()) as u16,
            gdt_ptr: gdt as u32,
        }
    }
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
            data: [zero_seg, code_segment, data_segment,video_segment]
        }
    }
    pub fn compose_gdtr(&'static self) -> GDTR {
        GDTR::new(self)
    }
    /**
     * 加载全局描述符表到GDTR寄存器
     */
    pub fn load_gdtr(&'static self) {
        load_gdtr_by_addr(self);
    }
    
    /**
     * 根据类型，取出描述符
     */
    pub fn get_descriptor(&'static self, desc_type: DecriptorType) -> &'static SegmentDescritor {
        &self.data[desc_type as usize]
    }

    /**
     * 把描述符数据塞入到GDT
     */
    pub fn set_descriptor(&'static mut self, desc_type: DecriptorType, descriptor: SegmentDescritor) {
        self.data[desc_type as usize] = descriptor;
    }
}


