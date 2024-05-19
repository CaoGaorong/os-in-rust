 /**
  * 段描述符 <https://en.wikipedia.org/wiki/Segment_descriptor>
  * 段描述符大小64位，8个字节。描述了一个内存段的属性，比如内存段的起始地址、段的大小、类型、特权级等属性
  */
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SegmentDescriptor {
    /**
     * 段界限低16位
     */
    seg_limit_16: u16,
    /**
     * 段基址低16位
     */
    seg_base_16: u16,
    /**
     * 段基址[16, 24)位
     */
    seg_base_24: u8,
    /**
     * type(4 bits) + S(1 bit) + DPL(2 bits) + p(1 bit)
     */
    type_s_dpl_p: u8,
    /**
     * 段界限[16, 20)位 + avl(1 bit) + L (1 bit) + D/B(1 bit) + G(1 bit)
     */
    seg_limit_20_avl_l_db_g: u8,
    /**
     * 段基址[24, 32)位
     */
    seg_base_32: u8
}
impl SegmentDescriptor {
    /**
     * 构建一个段描述符
     * seg_base_addr: 段基址，32位
     * seg_limit：段界限20位
     * granularity：段粒度
     * dpl：段权限
     * present：是否在内存中存在
     * seg_type： 段类型
     * avl：是否可用
     * l: 是否长模式64位
     * db：偏移地址和操作数大小是否为32位
     * <https://wiki.osdev.org/Global_Descriptor_Table>
     */
    pub const fn new(seg_base_addr: u32, 
                    seg_limit: u32, 
                    granularity: Granularity, 
                    dpl: SegmentDPL, 
                    present: bool, 
                    seg_type: SegmentType, 
                    avl: bool, 
                    l: bool, 
                    db: bool
        ) -> Self{
        let type_s_val = ((seg_type as u8) & 0b11111) as u8;
        let dpl_val = dpl as u8;
        let p_val:u8 = if present {1} else {0};

        let seg_limit_16_20_val = ((seg_limit & 0b111110000000000000000) >> 16) as u8;
        let avl_val:u8 = if avl {1} else {0};
        let l_val:u8 = if l {1} else {0};
        let db_val:u8 = if db {1} else {0};
        Self {
            seg_limit_16: seg_limit as u16,
            seg_base_16: seg_base_addr as u16,
            seg_base_24: (seg_base_addr >> 16) as u8,
            type_s_dpl_p: type_s_val | (dpl_val << 5) | (p_val << 7),
            seg_limit_20_avl_l_db_g: seg_limit_16_20_val | (avl_val << 4) | (l_val << 5) | (db_val <<6) | (granularity.value << 7),
            seg_base_32: (seg_base_addr >> 24) as u8,
        }
    }
    pub const fn empty() -> Self {
        Self {
            seg_limit_16: 0,
            seg_base_16: 0,
            seg_base_24: 0,
            type_s_dpl_p: 0,
            seg_limit_20_avl_l_db_g: 0,
            seg_base_32: 0,
        }
    }
}
/**
 * 段界限的粒度
 */
#[derive(Clone, Copy)]
pub struct Granularity {
    value: u8,
    unit: u32
}
impl Granularity {
    pub const fn new(g: GranularityEnum) -> Self {
        match g {
            GranularityEnum::UnitB => Granularity{value: 0, unit: 1},
            GranularityEnum::Unit4KB => Granularity{value: 1, unit: 4 * 1024},

        }
    }
}
pub enum GranularityEnum {
    /**
     * 粒度是1字节
     * 值为0，单位是1字节
    */
    UnitB,
    /**
     * 粒度是4KB
     * 值为1，单位是4*1024字节
     */
    Unit4KB,
}



/**
 * 段描述符的DPL
 */
pub enum SegmentDPL {
    LEVEL0 = 0,
    LEVEL1 = 1,
    LEVEL2 = 2,
    LEVEL3 = 3,
}

/**
 * 段的类型
 * <https://wiki.osdev.org/Global_Descriptor_Table>
 */
pub enum SegmentType {
    /**
     * 普通代码段
     * 1: s字段；非系统段
     * 1: e字段；可执行代码段
     * 0: dc字段；非一致性代码段
     * 0: rw字段；不可读写段
     * 0: a字段；
     */
    NormalCodeSegment = 0b11000,
    /**
     * 普通数据段
     * 1: s字段；非系统段 
     * 0: e字段；不可执行
     * 0: dc字段；非一致性代码段
     * 1: rw字段；可读写数据段
     * 0: a字段
     */
    NormalDataSegment = 0b10010,

    /**
     * TSS不忙的类型
     * 0: s字段，系统段
     * 1: 固定
     * 0: 固定
     * 0: 不忙
     * 1: 固定
     */
    TssNonBusySegment = 0b01001,

    /**
     * TSS不忙的类型
     * 0: s字段，系统段
     * 1: 固定
     * 0: 固定
     * 1: 忙
     * 1: 固定
     */
    TssBusySegment = 0b01011,
}

