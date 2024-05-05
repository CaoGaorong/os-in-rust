/**
 * 系统启动的上下文，在多个stage之间穿梭
 */
#[repr(C)]
#[derive(Clone, Copy)]
pub struct BootContext {
    /**
     * 内存图的长度
     */
    pub memory_map_addr: u32,
    /**
     * 内存图的长度
     */
    pub memory_map_len: u32,
}
impl BootContext {
    pub const fn empty() -> Self {
        Self {
            memory_map_addr: 0x1,
            memory_map_len: 0x1,
        }
    }
}