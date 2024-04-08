/**
 * 
 * 
 */

 
 /**
  * 段描述符 <https://en.wikipedia.org/wiki/Segment_descriptor>
  * 段描述符大小64位，8个字节。描述了一个内存段的属性，比如内存段的起始地址、段的大小、类型、特权级等属性
  */
#[repr(C)]
pub struct SegmentDescritor {
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
impl SegmentDPL {
    pub fn new(seg_base_addr: u32, seg_limit: u32, granularity: SegmentLimitGranularity, dpl: SegmentDPL, )
}
/**
 * 段界限的粒度
 */
enum SegmentLimitGranularity {
    /**
     * 粒度是1字节
     * 值为0，单位是1字节
    */
    UnitB = Granularity{value: 0, unit: 1},
    /**
     * 粒度是4KB
     * 值为1，单位是4*1024字节
     */
    Unit4KB = Granularity{value: 1, unit: 4 * 1024},
}

pub struct Granularity {
    value: u8,
    unit: u8
}

fn main() {
    let s = Granularity{value: 0, unit: 1};

}

/**
 * 段描述符的DPL
 */
enum SegmentDPL {
    LEVEL0 = 0,
    LEVEL1 = 1,
    LEVEL2 = 2,
    LEVEL3 = 3,
}

