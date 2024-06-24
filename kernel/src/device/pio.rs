use os_in_rust_common::{port::{self, Port}, printkln, utils, ASSERT, MY_PANIC};

use super::constant;

/**
 * PIO stands for Programed Input Output
 * PIO是一种内存和I/O设备数据交换的一种工作模式，包含：
 *  - 程序查询
 *  - 程序中断
 * 另一种符合ATA标准设备的工作模式是DMA
 * 关于PIO可以看：<https://wiki.osdev.org/ATA_PIO_Mode>
 */


/**
 * 从某一个把数据写入以port_base为端口起始的某个命令寄存器
 */
pub fn write_to_register(port_base: u16, register: CommandBlockRegister) {
    
    match register {
        CommandBlockRegister::Data(buf, bytes) => {
            let port = constant::DATA_REGISTER_OFFSET + port_base;
            // 从缓冲区中，取出2字节（一个字）的数据
            // let data = unsafe { *(buf as *const _ as *const u16) };
            // Port::<u16>::new(port).write(data);
            // 一次性读取多个字节
            port::write_words(port, buf.as_ptr() as u32, bytes / 2);

        },
        CommandBlockRegister::Feature(feature) => {
            let port = constant::FEATURE_REGISTER_OFFSET + port_base;
            // printkln!("write to port: 0x{:x}, feature: 0b{:b}", port, feature);
            Port::<u8>::new(port).write(feature);
        },
        CommandBlockRegister::SectorCount(sector_cnt) => {
            let port = constant::SECTOR_COUNT_REGISTER_OFFSET + port_base;
            // printkln!("write to port: 0x{:x}, sector_cnt: 0b{:b}", port, sector_cnt);
            Port::<u8>::new(port).write(sector_cnt);
        },
        CommandBlockRegister::LBALow(lba_low) => {
            let port = constant::LBA_LOW_REGISTER_OFFSET + port_base;
            // printkln!("write to port: 0x{:x}, lba_low: 0b{:b}", port, lba_low);
            Port::<u8>::new(port).write(lba_low);
        },
        CommandBlockRegister::LBAMid(lba_mid) => {
            let port = constant::LBA_MID_REGISTER_OFFSET + port_base;
            // printkln!("write to port: 0x{:x}, lba_mid: 0b{:b}", port, lba_mid);
            Port::<u8>::new(port).write(lba_mid);
        },
        CommandBlockRegister::LBAHigh(lba_high) => {
            let port = constant::LBA_HIGH_REGISTER_OFFSET + port_base;
            // printkln!("write to port: 0x{:x}, lba_high: 0b{:b}", port, lba_high);
            Port::<u8>::new(port).write(lba_high);
        },
        CommandBlockRegister::Device(device) => {
            let port = constant::DEVICE_REGISTER_OFFSET + port_base;
            // printkln!("write to port: 0x{:x}, device.data: 0b{:b}", port, device.data);
            Port::<u8>::new(port).write(device.data);
        },
        CommandBlockRegister::Command(command) => {
            let port = constant::COMMAND_REGISTER_OFFSET + port_base;
            // printkln!("write to port: 0x{:x}, command.data: 0x{:x}", port, command.data);
            Port::<u8>::new(port).write(command.data);
        },
        _ => {
            printkln!("{:?} register could not to write", register);
            MY_PANIC!("io command register error");
        }
    }
}

/**
 * 从某个以port_base为起始端口的寄存器中读取数据
 */
pub fn read_from_register(port_base: u16, register: CommandBlockRegister) {
    match register {
        CommandBlockRegister::Data(buf, bytes) => {
            let port = constant::DATA_REGISTER_OFFSET + port_base;
            // 开始批量读取。字节转成字。1字 = 2字节
            port::read_words(port, bytes / 2, buf.as_ptr() as u32)
        },
        CommandBlockRegister::Error(mut error) => {
            let port = constant::ERROR_REGISTER_OFFSET + port_base;
            let data = Port::<u8>::new(port).read();
            error.data = data;
        },
        CommandBlockRegister::RegularStatus(status) => {
            let port = constant::STATUS_REGISTER_OFFSET + port_base;
            let data = Port::<u8>::new(port).read();
            status.data = data;
        },
        _ => {
            printkln!("{:?} register could not read from", register);
            MY_PANIC!("io command register error");
        }
    }
}




#[derive(Debug)]
pub enum CommandBlockRegister<'a> {
    /**
     * data寄存器数据，port base + 0
     * &[u8] 把data寄存器的数据跟内存缓冲区交互
     * u32 是交互的字节数量
     */
    Data(&'a [u8], u32),
    /**
     * error寄存器数据 ，port base + 1
     */
    Error(&'a mut ErrorRegister),
    /**
     * feature寄存器数据 ，port base + 1
     */
    Feature(u8),
    /**
     * sector count寄存器 ，port base + 2
     */
    SectorCount(u8),
    /**
     * lba low寄存器数据 ，port base + 3
     */
    LBALow(u8),
    /**
     * lba mid寄存器数据 ，port base + 4
     */
    LBAMid(u8),
    /**
     * lba high寄存器数据 ，port base + 5
     */
    LBAHigh(u8),
    /**
     * device寄存器数据 ，port base + 6
     */
    Device(DeviceRegister),
    /**
     * status寄存器数据 ，port base + 7
     */
    RegularStatus(&'a mut StatusRegister),
    /**
     * command寄存器数据 ，port base + 7
     */
    Command(CommandRegister),
}

#[derive(Debug)]
#[repr(transparent)]
pub struct ErrorRegister {
    pub data: u8,
}
impl ErrorRegister {
    pub fn empty() -> Self {
        Self {
            data: 0,
        }
    }
}

/**
 * Device寄存器的结构
 * 仅写入
 */
#[derive(Debug)]
#[repr(transparent)]
pub struct DeviceRegister {
    data: u8
}
impl DeviceRegister {
    pub fn new(lba: u8, primary: bool, is_lba: bool) -> Self {
        Self {
            data: 
                (lba & 0b1111)  |
                // 0代表主盘，1代表从盘
                ((if primary {0} else {1}) << 4) as u8|
                1 << 5 |
                (utils::bool_to_int(is_lba) << 6) as u8 |
                1 << 7
        }
    }
}


enum StatusRegisterBitEnum {
    /**
     * 标识有错误发生
     */
    Err = 0x0,
    /**
     * index. always zero
     */
    Idx = 0x1, 
    /**
     * Corrected data. always zero
     */
    Corr = 0x2,
    /**
     * Data request. ready to accept or transfer data
     */
    Drq = 0x3,
    /**
     * Service. 
     */
    Srv = 0x4,
    /**
     * Drive Fault error
     */
    Df = 0x5,
    /**
     * disk Ready or not
     */
    Rdy = 0x6,
    /**
     * disk busy or not
     */
    Bsy = 0x7
}

/**
 * status寄存器，
 * 仅读取
 */
#[derive(Debug)]
#[repr(transparent)]
pub struct StatusRegister {
    pub data: u8
}
impl StatusRegister {
    pub fn empty() -> Self {
        Self {
            data: 0
        }
    }
    /**
     * 是否等待数据请求（读取或者写入）
     */
    pub fn data_request(&self) -> bool {
        // 第三位设置为1了，那么就是等待数据请求
        self.is_set(StatusRegisterBitEnum::Drq)
    }

    /**
     * 是否处于忙的状态
     */
    pub fn busy(&self) -> bool {
        self.is_set(StatusRegisterBitEnum::Bsy)
    }

    /**
     * 该status寄存器的idx位是否被设置为1
     */
    fn is_set(&self, bit_enum: StatusRegisterBitEnum) -> bool {
        let idx = bit_enum as u8;
        (self.data & (1 << idx)) >> idx  == 0x1
    }

}
/**
 * 仅写入
 */
#[derive(Debug)]
#[repr(transparent)]
pub struct CommandRegister {
    data: u8
}

impl CommandRegister {
    pub fn new(command: PIOCommand) -> Self {
        Self {
            data: command as u8
        }
    }
}


#[derive(Debug)]
pub enum PIOCommand {
    // 识别硬盘命令
    Identify = 0xEC,
    // 读取硬盘命令
    Read = 0x20,
    // 写入硬盘命令
    Write = 0x30,
    // 刷新命令
    Flush = 0xE7,
}
