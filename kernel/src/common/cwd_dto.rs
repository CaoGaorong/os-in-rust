pub struct CwdDto<'a> {
    pub buff: &'a mut [u8],
    pub str: Option<&'a str>,
}