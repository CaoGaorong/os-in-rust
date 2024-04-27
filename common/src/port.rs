use core::{arch::asm, marker::PhantomData};

pub struct Port<T> {
    port: u16,
    phantom: PhantomData<T>
}

impl <T:PortRead + PortWrite> Port<T> {
    pub fn new(port: u16) -> Port<T> {
        Port {
            port,
            phantom: PhantomData,
        }
    }
}

impl <T:PortRead> Port<T> {
    pub fn read(&self) -> T {
        T::read_from_port(self.port)
    }
}

impl <T:PortWrite> Port<T> {
    pub fn write(&self, value: T) {
        T::write_to_port(self.port, value)
    }
}


/**
 * 定义写端口的接口
 */
trait PortWrite {
    fn write_to_port(port: u16, value: Self);
}

/**
 * 定义读数据的接口
 */
trait PortRead {
    fn read_from_port(port: u16) -> Self;
}

impl PortRead for u8 {
    fn read_from_port(port: u16) -> Self {
        let value: u8;
        unsafe {
            asm!("in al, dx", in("dx") port, out("al") value);
        }
        value
    }
}
impl PortRead for u16 {
    fn read_from_port(port: u16) -> Self {
        let value: u16;
        unsafe {
            asm!("in ax, dx", in("dx") port, out("ax") value);
        }
        value
    }
}

impl PortWrite for u8 {
    fn write_to_port(port: u16, value: Self) {
        unsafe {
            asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
        }
    }
}
impl PortWrite for u16 {
    fn write_to_port(port: u16, value: Self) {
        unsafe {
            asm!("out dx, ax", in("dx") port, in("ax") value, options(nomem, nostack, preserves_flags));
        }
    }
}
