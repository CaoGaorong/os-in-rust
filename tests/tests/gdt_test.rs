
#[cfg(test)]
mod tests {
    use std::slice;

    use os_in_rust_common::gdt::GlobalDecriptorTable;
    use os_in_rust_common::gdt::GDTR;
    use os_in_rust_common::sd;
    use os_in_rust_common::gdt;

    #[test]
    fn left_shift() {
        let a = 0b0;
        let b = 0b1;
        let c = 0b1;
        assert_eq!(0b101, c << 2 | a << 1 | b);
    }
    
    #[test]
    fn right_shift() {
        let long = 0b11110000;
        // 取低4位
        assert_eq!(0b0000, long & 0b1111);
        assert_eq!(0b1111, long >> 4);
    }

    #[test]
    fn segement_decriptor() {
        let base_addr:u32 = 0x00000000;
        let seg_limit = 0xfffff;
        let granularity = sd::Granularity::new(sd::GranularityEnum::Unit4KB);
        // 代码段
        let code_segment = sd::SegmentDescritor::new(
            base_addr, 
            seg_limit, 
            granularity, 
            sd::SegmentDPL::LEVEL0, 
            true, 
            sd::SegmentType::NormalCodeSegment, 
            false, 
            false, 
            true
        );
        let code_segment_data = unsafe{(*(&code_segment as *const sd::SegmentDescritor as *const u64)) as u64};
        println!("0x{:x}", code_segment_data);
        // 我期待的代码段
        let expected_code_seg:u64 = 0b00000000110011111001100000000000<<32 | 0x0000FFFF;
        println!("0x{:x}", expected_code_seg);
        assert_eq!(expected_code_seg, code_segment_data);
    }

    static  gdt: GlobalDecriptorTable = gdt::GlobalDecriptorTable::new();
    #[test] 
    fn video_seg_test() {
        let segment = gdt.video_seg;
        let segment_data = unsafe{(*(&segment as *const sd::SegmentDescritor as *const u64)) as u64};
        println!("0x{:x}", segment_data);
    }
    #[test]
    fn gdt_test() {

        println!("gdt address:{:p}", (&gdt as *const GlobalDecriptorTable));
        let gdtr =  gdt.compose_gdtr();
        let pointer = &gdtr as *const GDTR as *const u8;
        println!("0x{:x}", unsafe{*(pointer.offset(0))});
        println!("0x{:x}", unsafe{*(pointer.offset(1))});
        println!("0x{:x}", unsafe{*(pointer.offset(2))});
        println!("0x{:x}", unsafe{*(pointer.offset(3))});
        println!("0x{:x}", unsafe{*(pointer.offset(4))});
        println!("0x{:x}", unsafe{*(pointer.offset(5))});
    }
}
