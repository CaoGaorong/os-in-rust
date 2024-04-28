use os_in_rust_common::sd::SegmentDescriptor;
use os_in_rust_common::gdt;
use os_in_rust_common::reg_cr0;
use os_in_rust_common::selector;
use os_in_rust_common::instruction;


pub fn enter_protect_mode() {
    // 禁用中断
    instruction::disable_interrupt();

    // 加载GDT到GDTR寄存器中
    gdt::load_gdt();
    // 把PE位打开
    reg_cr0::set_on(reg_cr0::CR0::PE);
    // 加载数据段选择子
    selector::load_data_selector();

    // 把中断不打开，不然屏幕一直闪缩
    // 启用中断
    // interrupt::enable_interrupt();
}
