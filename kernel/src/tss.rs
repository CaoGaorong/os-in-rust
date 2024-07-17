use core::{arch::asm, mem::size_of};

use os_in_rust_common::{gdt::{self, DescriptorType}, printkln, racy_cell::RacyCell, sd::{Granularity, GranularityEnum, SegmentDPL, SegmentDescriptor, SegmentType}, selector::SegmentSelector};

/**
 * TSS的结构：<https://wiki.osdev.org/Task_State_Segment>
 * - LINK: Previous Task Link Field. Contains the Segment Selector for the TSS of the previous task.
 * - SS0, SS1, SS2: The Segment Selectors used to load the stack when a privilege level change occurs from a lower privilege level to a higher one.
 * - ESP0, ESP1, ESP2: The Stack Pointers used to load the stack when a privilege level change occurs from a lower privilege level to a higher one.
 * - IOPB: I/O Map Base Address Field. Contains a 16-bit offset from the base of the TSS to the I/O Permission Bit Map.
 * - SSP: Shadow Stack Pointer.
 */
#[repr(C, packed)]
pub struct Tss {
    /**
     * 上一个任务的指针，这个字段是上个任务的TSS选择子
     * Previous Task Link Field. Contains the Segment Selector for the TSS of the previous task.
     */
    previous_link: u16,
    link_reserved: u16,

    esp0: u32,
    
    ss0: u16,
    ss0_reserved: u16,
    
    esp1: u32,
    
    ss1: u16,
    ss1_reserved: u16,
    
    esp2: u32,
    
    ss2: u16,
    ss2_reserved: u16,

    cr3: u32,
    
    eip: u32,
    
    eflags: u32,
    eax: u32,
    ecx: u32,
    edx: u32,
    ebx: u32,
    esp: u32,
    ebp: u32,
    esi: u32,
    edi: u32,
    
    es: u16,
    es_preserved: u16,

    cs: u16,
    cs_preserved: u16,

    ss: u16,
    ss_preserved: u16,

    ds: u16,
    ds_preserved: u16,

    fs: u16,
    fs_preserved: u16,

    gs: u16,
    gs_preserved: u16,

    ldtr: u16,
    lgdtr_reserved: u16,

    iopb_reserved: u16,
    lopb: u32,
}
impl Tss {
    pub const fn empty() -> Self {
        Self {
            previous_link: 0,
            link_reserved: 0,
            esp0: 0,
            ss0: 0,
            ss0_reserved: 0,
            esp1: 0,
            ss1: 0,
            ss1_reserved: 0,
            esp2: 0,
            ss2: 0,
            ss2_reserved: 0,
            cr3: 0,
            eip: 0,
            eflags: 0,
            eax: 0,
            ecx: 0,
            edx: 0,
            ebx: 0,
            esp: 0,
            ebp: 0,
            esi: 0,
            edi: 0,
            es: 0,
            es_preserved: 0,
            cs: 0,
            cs_preserved: 0,
            ss: 0,
            ss_preserved: 0,
            ds: 0,
            ds_preserved: 0,
            fs: 0,
            fs_preserved: 0,
            gs: 0,
            gs_preserved: 0,
            ldtr: 0,
            lgdtr_reserved: 0,
            iopb_reserved: 0,
            lopb: size_of::<Tss>() as u32,
        }
    }
}

/**
 * 加载TSS到TR寄存器
 * tss_selector: tss所在GDT的选择子
 */
#[cfg(all(not(test), target_arch = "x86"))]
pub fn load_tss(tss_selector: u16) {
    unsafe {
        asm!(
            "ltr {:x}",
            in(reg) tss_selector
        )
    }
}
#[cfg(all(not(target_arch = "x86")))]
pub fn load_tss(tss_selector: u16) {
    todo!()
}

pub static GLOBAL_TSS: RacyCell<Tss> = RacyCell::new(Tss::empty());

#[inline(never)]
pub fn tss_init() {
    let global_tss = unsafe { GLOBAL_TSS.get_mut() };
    // ss0就是数据段选择子
    global_tss.ss0 = SegmentSelector::Data0Selector as u16;

    // 构造tss描述符
    let tss_descriptor = SegmentDescriptor::new(
        global_tss as *const _ as u32, 
        size_of::<Tss>().try_into().unwrap(), 
        Granularity::new(GranularityEnum::Unit4KB), 
        SegmentDPL::LEVEL0, 
        true, 
        SegmentType::TssNonBusySegment, 
        false, 
        false, 
        false, // 固定为0
    );
    // 设置TSS描述符
    gdt::set_descriptor(DescriptorType::Tss, tss_descriptor);

    // 用户进程的代码段选择子
    let user_code_descriptor = SegmentDescriptor::new(
        0, 
        0xfffff,  // 段界限，20位
        Granularity::new(GranularityEnum::Unit4KB), 
        SegmentDPL::LEVEL3, 
        true, 
        SegmentType::NormalCodeSegment, 
        false,
        false,
        true, // 固定为0
    );
    // 设置代码段选择子项
    gdt::set_descriptor(DescriptorType::UserCode, user_code_descriptor);


    // 用户进程的数据段选择子
    let user_data_descriptor = SegmentDescriptor::new(
        0, 
        0xfffff,  // 段界限，20位
        Granularity::new(GranularityEnum::Unit4KB), 
        SegmentDPL::LEVEL3, 
        true, 
        SegmentType::NormalDataSegment, 
        false,
        false,
        true, // 固定为0
    );
    // 设置数据段选择子项
    gdt::set_descriptor(DescriptorType::UserData, user_data_descriptor);


    // 重新加载一下GDT
    gdt::load_gdt();
    // tss选择子可以访问到GDT中TSS，把这个tss选择子加载到tr寄存器
    load_tss(SegmentSelector::TssSelector as u16);
}

/**
 * 更新全局TSS的esp0
 */
pub fn update_esp0(new_esp0: u32) {
    unsafe { GLOBAL_TSS.get_mut().esp0 = new_esp0 };
}