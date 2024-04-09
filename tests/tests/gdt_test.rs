
#[cfg(test)]
mod tests {
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
        let base_addr:u32 = 0xffff0000;
        let seg_limit = 0xff;
        let granularity = gdt::Granularity::new(gdt::GranularityEnum::Unit4KB);
        let dpl = gdt::SegmentDPL::LEVEL0;
        gdt::SegmentDescritor::new(base_addr, seg_limit, granularity, dpl, present, seg_type, avl, l, db)
    }
}
