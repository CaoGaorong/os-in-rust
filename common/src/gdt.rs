use core::mem::size_of;

use crate::sd::{Granularity, GranularityEnum, SegmentDPL, SegmentDescritor, SegmentType};

static GDT: GlobalDecriptorTable = GlobalDecriptorTable::new();

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
pub struct GDTR {
    /**
     * 指向全局描述符表的起始地址
     */
    gdt_ptr: *const GlobalDecriptorTable,
    /**
     * 全局描述符表的大小。表中元素的个数
     */
    gdt_limit: u16,
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
            gdt_ptr: self as *const GlobalDecriptorTable,
            gdt_limit: (size_of::<GlobalDecriptorTable>() / size_of::<SegmentDescritor>()) as u16
        }
    }
}
