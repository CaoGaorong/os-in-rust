
use crate::{constants, idt::InterruptTypeEnum, port::Port, println, utils};
// 设置起始的中断号
static  START_REQ_NO:u8 = constants::INTERRUPT_NO_START;
// 设置级联在主片的索引，也就是REQ_NO
static  CHAINED_INDEX:u8 = 0x2;

// 把两个片级联
static CHAINED_PICS:ChainedPics = ChainedPics::new(
        Pic::new(START_REQ_NO, Port::<u8>::new(PicPort::PrimaryCommand as u16), Port::<u8>::new(PicPort::PrimaryData as u16)),
      Pic::new(START_REQ_NO + 0x8, Port::<u8>::new(PicPort::SecondaryCommand as u16), Port::<u8>::new(PicPort::SecondaryData as u16)), 
    CHAINED_INDEX
);


/**
 * PIC的控制端口
 * <https://wiki.osdev.org/8259_PIC#Programming_with_the_8259_PIC>
 */
enum PicPort {
    // 主片的控制器端口
    PrimaryCommand = 0x20,
    // 主片的数据寄存器端口
    PrimaryData = 0x21,
    // 从片的控制寄存器端口
    SecondaryCommand = 0xA0,
    // 主片的数据寄存器端口
    SecondaryData = 0xA1,
}

pub fn pic_init() {
    // 初始化所有的配置
    CHAINED_PICS.init();
}

pub fn send_end_of_interrupt() {
    CHAINED_PICS.send_end_of_interrupt();
}

/**
 * 级联的PIC
 */
pub struct ChainedPics {
    /**
     * 主片
     */
    pub primary: Pic,
    /**
     * 从片
     */
    pub secondary: Pic,

    /**
     * 级联在主片的那个接口下标
     */
    chained_idx: u8,
}
impl ChainedPics {
    const fn new(primary: Pic, secondary: Pic, chained_idx: u8) -> Self {
        Self {
            primary, secondary, chained_idx
        }
    }
    /**
     * 对中断控制器的操作Programable Interrupt Controller
     * 级联的结构：
     *                     ____________                          ____________
     * Real Time Clock --> |            |   Timer -------------> |            |
     * ACPI -------------> |            |   Keyboard-----------> |            |      _____
     * Available --------> | Secondary  |----------------------> | Primary    |     |     |
     * Available --------> | Interrupt  |   Serial Port 2 -----> | Interrupt  |---> | CPU |
     * Mouse ------------> | Controller |   Serial Port 1 -----> | Controller |     |_____|
     * Co-Processor -----> |            |   Parallel Port 2/3 -> |            |
     * Primary ATA ------> |            |   Floppy disk -------> |            |
     * Secondary ATA ----> |____________|   Parallel Port 1----> |____________|
     */
    fn init(&self) {
        let wait_port = Port::<u8>::new(0x80);
        // let wait = || wait_port.write(0x0);
        let wait = || ();

        let primary_mask = self.primary.data_port.read();
        let secondary_mask = self.secondary.data_port.read();
        
        // 写入ICW1 0x20 0x11
        let icw1 = ICW1::new(true, false, false);
        self.primary.command_port.write(icw1.data);
        wait();

        // 0xA0 0x11
        self.secondary.command_port.write(icw1.data);
        wait();

        // 写入ICW2 0x20 0x21
        self.primary.data_port.write(ICW2::new(*&self.primary.start_int_no).data);
        wait();

        // 0xA1 0x28
        self.secondary.data_port.write(ICW2::new(*&self.secondary.start_int_no).data);
        wait();

        // ICW3， 级联在主片的第idx下标处，所以主片这一位设置为1
        // 0x21 0x04
        self.primary.data_port.write(ICW3::new(1 << self.chained_idx).data);
        wait();
        // 0xA1 0x02
        self.secondary.data_port.write(ICW3::new(self.chained_idx).data);
        wait();

        // ICW4 0x21 0x01
        let icw4 = ICW4::new(true, false, false, false);
        wait();
        self.primary.data_port.write(icw4.data);
        wait();
        self.secondary.data_port.write(icw4.data);

        // 主片，打开时钟中断、键盘中断、以及从片的中断
        self.primary.data_port.write(OCW1::new(0b11111100).data);
        wait();
        // 打开从片的硬盘中断
        self.secondary.data_port.write(OCW1::new(0b11111111).data);
        wait();

        // self.primary.data_port.write(primary_mask);
        // self.secondary.data_port.write(secondary_mask);

    }

    pub fn send_end_of_interrupt(&'static self) {
        self.primary.end_of_int();
        self.secondary.end_of_int();
    }

}


struct Pic {
    /**
     * 起始的中断号
     */
    start_int_no: u8,
    /**
     * 命令寄存器的端口
     */
    command_port: Port<u8>,
    /**
     * 数据寄存器的端口
     */
    data_port: Port<u8>
}

impl Pic {
    const fn new(start_int_no: u8, command_port: Port<u8>, data_port: Port<u8>) -> Self {
        Self {
            start_int_no, command_port, data_port
        }
    }

    /**
     * 发送EOI
     */
    fn end_of_int(&self) {
        self.command_port.write(0x20);
    }
}







#[repr(C, packed)]
struct ICW1 {
    data: u8
}
impl ICW1 {
    /**
     * ic4: 是否初始化ICW4
     * single: 是否单片；1：单片；0：级联
     * trigger_mode: 1：电平触发；0：边沿触发
     */
    fn new(ic4:bool, single: bool, trigger_mode: bool) -> Self {
        Self {
            data: 
                utils::bool_to_u8(ic4) |
                utils::bool_to_u8(single) << 1 |
                0 << 2 |
                utils::bool_to_u8(trigger_mode) << 3 |
                1<< 4 // 固定是1
        }
    }
}

/**
 * ICW2用于设置起始中断号
 */
#[repr(C, packed)]
struct ICW2 {
    data: u8
}
impl ICW2 {
    /**
     * start_int_no起始中断号
     */
    fn new(start_int_no: u8) -> Self {
        Self {
            data: start_int_no
        }
    }
}

/**
 * ICW2用于设置级联情况，主片和从片哪个端口相连
 */
#[repr(C, packed)]
struct ICW3 {
    data: u8
}
impl ICW3 {

    fn new(data: u8) -> Self {
        Self {
            data
        }
    }
    /**
     * 对于主片，要设置好哪个口连上了从片
     * connect_idx 连接了从片的端口下标【0-8】
     */
    fn primary(connect_idx: u8) -> Self {
        Self {
            data: 1 << connect_idx
        }
    }

    /**
     * 对于从片，需要指定自己是与主片的哪个口连的
     * primary_idx: 当前从片，与主片的primary_idx这个口连接
     */
    fn secondary(primary_idx: u8) -> Self {
        Self {
            data: primary_idx
        }
    }
}

/**
 * ICW2用于设置控制器的工作方式
 */
#[repr(C, packed)]
struct ICW4 {
    data: u8
}
impl ICW4 {
    /**
     * ICW4的配置
     *  - x86: 0：8085以及更老的；1：x86
     *  - auto_end: 收到中断信息号后，是否自动结束
     *  - buffer: 是否缓冲模式
     *  - full_nested: 是否全嵌套模式
     */
    fn new(x86: bool, auto_end: bool, buffer: bool, full_nested: bool) -> Self {
        Self {
            data: 
                utils::bool_to_u8(x86) |
                utils::bool_to_u8(auto_end) << 1 |
                utils::bool_to_u8(buffer) << 2 | 
                utils::bool_to_u8(full_nested) << 3
        }
    }
}


/**
 * OCW1用于屏蔽8258A的中断信号
 */
#[repr(C, packed)]
struct OCW1 {
    data: u8
}
impl OCW1 {
    /**
     * 要屏蔽的中断请求号
     * 一共8位，比如0b00000001，表示irq为0的那个端口的信号被屏蔽
     */
    fn new(irq_to_mask: u8) -> Self {
        Self {
            data: irq_to_mask
        }
    }
}

/**
 * OCW2设置各种属性
 */
#[repr(C, packed)]
struct OCW2 {
    data: u8
}
impl OCW2 {
    /**
     * rotation: 优先级是否循环
     * specific_level: 如果SL=1, R=1，那么L2、L1、L0可以指定某个IRQ接口优先级最低。
     * end_of_int: 结束中断；只有只有在ICW4中的AEOI为0才有效（即手动模式）。EOI为1, 8259A会将L0 ~ L2在ISR中对应的位清0。
     * config_data: 配合上面的设置项使用
     */
    fn new(rotation: bool, specific_level: bool, end_of_int: bool, config_data: u8) -> Self {
        Self {
            data: 
                (config_data & 0b111) |
                utils::bool_to_u8(rotation) << 7 |
                utils::bool_to_u8(specific_level) << 6 |
                utils::bool_to_u8(end_of_int) << 5
        }
    }
}
