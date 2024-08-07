use core::ptr;

use os_in_rust_common::{constants, cstring_utils, domain::LbaAddr, elem2entry, linked_list::LinkedNode, printk, printkln, ASSERT, MY_PANIC};

use crate::sync::{Lock, Semaphore};

use super::pio::{self, CommandBlockRegister, CommandRegister, DeviceRegister, PIOCommand, StatusRegister};

/**
 * 本文件是关于IDE硬盘的通道的相关结构定义
 * 可以见 <https://wiki.osdev.org/ATA_PIO_Mode>
 * 
 */

/**
 * ide通道的逻辑结构
 * 关于ide通道，寄存器可以见：<https://wiki.osdev.org/ATA_PIO_Mode#Registers>
 * 
 * 一个机器可以挂在2个通道（主从），每个通道可以挂在2个硬盘（主从）。每个通道占据一个8259的中断端口
 */
#[derive(Debug)]
pub struct ATAChannel {
    /**
     * IDE通道名称
     * 以C字符串的格式存储
     */
    pub name: [u8; constants::IDE_CHANNEL_NAME_LEN],
    /**
     * 本通道的起始端口号
     * 通道用到的硬盘控制器的寄存器，都是根据这个起始端口号递增的，
     * 详情见：<https://wiki.osdev.org/ATA_PIO_Mode#Registers>
     */
    pub port_base: u16,

    /**
     * 该通道用的中断号。1个通道占据一个中断端口，有一个中断号
     */
    pub irq_no: u8,

    /**
     * 是否正在等待中断
     */
    pub expecting_intr: bool,

    /**
     * 同步锁
     */
    lock: Lock,

    /**
     * 使用信号量阻塞自己。当硬盘完成中断后产生的中断唤醒自己
     */
    pub disk_done: Semaphore,

    /**
     * 一个通道可以挂在两个硬盘
     */
    pub disks: [Option<Disk>; constants::DISK_CNT_PER_CHANNEL],
}


impl ATAChannel {
    pub const fn empty() -> Self {
        const ARRAY_REPEAT_VALUE: Option<Disk> = None;
        Self {
            name: [0; constants::IDE_CHANNEL_NAME_LEN],
            port_base: 0,
            irq_no: 0,
            expecting_intr: false,
            lock: Lock::new(),
            disk_done: Semaphore::new(0),
            disks: [ARRAY_REPEAT_VALUE; constants::DISK_CNT_PER_CHANNEL],
        }
    }

    pub fn new(name: &[u8], port_base: ChannelPortBaseEnum, irq_no: ChannelIrqNoEnum) -> Self {
        ASSERT!(name.len() >= constants::IDE_CHANNEL_NAME_LEN);
        let mut name_buf = [0; constants::IDE_CHANNEL_NAME_LEN];
        name_buf.copy_from_slice(&name[0 .. constants::IDE_CHANNEL_NAME_LEN]);
        const ARRAY_REPEAT_VALUE: Option<Disk> = None;
        Self {
            name: name_buf,
            port_base: port_base as u16,
            irq_no: irq_no as u8,
            expecting_intr: false,
            lock: Lock::new(),
            disk_done: Semaphore::new(0),
            disks: [ARRAY_REPEAT_VALUE; constants::DISK_CNT_PER_CHANNEL],
        }
    }


    pub fn channel_ready(&mut self) {
        if !self.expecting_intr {
            printkln!("inter ignored");
            return;
        }
        self.expecting_intr = false;
        // 唤醒等待的线程
        self.disk_done.up();
        let mut status_register = StatusRegister::empty();
        pio::read_from_register(self.port_base, CommandBlockRegister::RegularStatus(&mut status_register));

    }
    pub fn get_name(&self) -> &str {
        let name = cstring_utils::read_from_bytes(&self.name);
        ASSERT!(name.is_some());
        name.expect("invalid name")
    }

}
/**
 * 一个硬盘的结构
 */
#[derive(Debug)]
pub struct Disk {
    /**
     * 硬盘的名称
     * 以C字符串的格式存储
     */
    pub name: [u8; constants::DISK_NAME_LEN],
    /**
     * 该硬盘归属的通道
     */
    // from_channel: Option<&'static mut ATAChannel>,
    pub from_channel: *mut ATAChannel,

    /**
     * 是否是主硬盘
     */
    pub primary: bool,

    /**
     * 一个硬盘最多4个主分区
     */
    pub primary_parts: [Option<Partition>; constants::DISK_PRIMARY_PARTITION_CNT],

    /**
     * 逻辑分区。理论上一个硬盘无限多个逻辑分区数量
     */
    pub logical_parts: [Option<Partition>; constants::DISK_LOGICAL_PARTITION_CNT],
}

impl Disk {
    pub const fn empty() -> Self {
        const ARRAY_REPEAT_VALUE: Option<Partition> = None;
        Self {
            name:  [0; constants::DISK_NAME_LEN],
            from_channel: ptr::null_mut(),
            primary: false,
            primary_parts: [ARRAY_REPEAT_VALUE; 4],
            logical_parts: [ARRAY_REPEAT_VALUE; constants::DISK_LOGICAL_PARTITION_CNT],
        }
    }

    pub fn new(name: &[u8], from_channel:  *mut ATAChannel, primary: bool) -> Self {
        ASSERT!(name.len() >= constants::DISK_NAME_LEN);
        let mut name_buf = [0; constants::DISK_NAME_LEN];
        name_buf.copy_from_slice(&name[0 .. constants::DISK_NAME_LEN]);
        const ARRAY_REPEAT_VALUE: Option<Partition> = None;
        Self {
            name: name_buf,
            from_channel: from_channel,
            primary: primary,
            primary_parts: [ARRAY_REPEAT_VALUE; constants::DISK_PRIMARY_PARTITION_CNT],
            logical_parts: [ARRAY_REPEAT_VALUE; constants::DISK_LOGICAL_PARTITION_CNT],
        }
    }

    pub fn get_name(&self) -> &str {
        let name = cstring_utils::read_from_bytes(&self.name);
        ASSERT!(name.is_some());
        name.expect("invalid name")
    }

    pub fn identify(&mut self) {
        // 该硬盘归属的通道
        self.select_disk();
        // 发送identify 命令
        self.set_command(PIOCommand::Identify);
        
        // 阻塞
        self.block();

         // 检查硬盘是否准备好了
         if !self.ready_for_read() {
            printkln!("failed to read disk. disk not ready, disk name: {}", self.get_name());
            MY_PANIC!("disk status error");
        }

        // 读取出数据
        let buf = [0; constants::DISK_SECTOR_SIZE];
        self.read_bytes(&buf, constants::DISK_SECTOR_SIZE.try_into().unwrap());

        // 把读取到的结果，转成固定格式
        let identify_res =  unsafe { &*(&buf as *const _ as *const DiskIdentifyContent) };

        let disk_name = self.get_name();
        let sn_name = core::str::from_utf8(&identify_res.sn).expect("Invalid name");
        let module_name = core::str::from_utf8(&identify_res.module).expect("invalid moduel name");
        // printkln!("disk info: {},  sn: {}", disk_name, sn_name);
        // printkln!("module: {}", module_name);
        // printkln!("disk sector count: {}", identify_res.sec_cnt as u32);
    }
    

    /**
     * 从lba_start为起始地址的扇区中，读取连续sec_cnt扇区的数据，到buf缓冲区中
     */
    #[inline(never)]
    pub fn read_sectors(&mut self, lba_start: LbaAddr, sec_cnt: usize, buf: &mut [u8]) {
        let lba_start = lba_start.get_lba() as usize;
        let lba_end = lba_start + sec_cnt;
        if lba_end > (constants::DISK_MAX_SIZE / constants::DISK_SECTOR_SIZE as u64) as usize {
            printkln!("error to read sector. exceed maximum sector. lba:{}, sec_cnt:{}", lba_start, sec_cnt);
            MY_PANIC!("");
        }
        if buf.len() < sec_cnt * constants::DISK_SECTOR_SIZE {
            printkln!("error to read sector. buffer capacity not enough. lba:{}, sec_cnt:{}, buf len:{}", lba_start, sec_cnt, buf.len());
            MY_PANIC!("");
        }

        // 锁住channel。1个通道无法并发执行
        self.lock_channel();

        // 一共读取sec_cnt个扇区，批量读取
        let step = 1;
        for lba in (lba_start .. lba_end).step_by(step) {
            // 每次读取sec_once个扇区
            let sec_once = step.try_into().unwrap();
            // let sec_once: u16 = if lba + step > lba_end {
            //     (lba_end - lba).try_into().unwrap()
            // } else {
            //     step.try_into().unwrap()
            // };


            // 设置好要读取的扇区
            self.set_op_sector(lba.try_into().unwrap(), sec_once);

            // 发送读取命令
            self.set_command(PIOCommand::Read);

            // 然后进入阻塞。等待硬盘就绪的中断信号
            self.block();

            // 检查硬盘是否准备好了
            if !self.ready_for_read() {
                printkln!("failed to read disk. disk not ready, lba:{}, sec_once:{}", lba, sec_once);
                MY_PANIC!("disk status error");
            }

            // 读取出多个字节，放到buf缓冲区中
            let sec_once_bytes = sec_once as u32 * constants::DISK_SECTOR_SIZE as u32;
            self.read_bytes(&buf[(lba - lba_start) * sec_once_bytes as usize ..], sec_once_bytes);
        }
        // 解锁channel
        self.unlock_channel();
    }

    /**
     * 把buf的数据写入到lba_start起始的地址 的连续 sec_cnt个扇区中
     */
    #[inline(never)]
    pub fn write_sector(&mut self, buf: &[u8], lba_start: LbaAddr, sec_cnt: usize) {
        let lba_start = lba_start.get_lba() as usize;
        let lba_end = lba_start + sec_cnt;
        if lba_end > (constants::DISK_MAX_SIZE / constants::DISK_SECTOR_SIZE as u64) as usize {
            printkln!("error to write sector. exceed maximum sector. lba:{}, sec_cnt:{}", lba_start, sec_cnt);
            MY_PANIC!("");
        }
        // 缓冲区的数据，只能多，不能少
        if buf.len() < sec_cnt * constants::DISK_SECTOR_SIZE {
            printkln!("error to write sector. buffer capacity not enough. lba:{}, sec_cnt:{}, buf len:{}", lba_start, sec_cnt, buf.len());
            MY_PANIC!("");
        }
        self.lock_channel();
        // 一共读取sec_cnt个扇区，批量
        let step = 1; // 如果使用qemu，最好只操作1个扇区
        for lba in (lba_start .. lba_end).step_by(step) {
            // 每次读取sec_once个扇区
            let sec_once = step.try_into().unwrap();
            // let sec_once: u16 = if lba + step > lba_end {
            //     (lba_end - lba).try_into().unwrap()
            // } else {
            //     step.try_into().unwrap()
            // };

            // 设置好要操作的扇区
            self.set_op_sector(lba as u32, sec_once);

            // 发送写入命令
            self.set_command(PIOCommand::Write);
            // 检查硬盘是否准备好了
            if !self.ready_for_read() {
                printkln!("failed to read disk. disk not ready, lba:{}, sec_once:{}", lba, sec_once);
                MY_PANIC!("disk status error");
            }

            // 读取出多个字节，放到buf缓冲区中
            let sec_once_bytes = sec_once as u32 * constants::DISK_SECTOR_SIZE as u32;
            self.write_bytes(&buf[(lba - lba_start) * sec_once_bytes as usize ..], sec_once_bytes);

            // 然后进入阻塞。等待硬盘就绪的中断信号
            self.block();
        }
        self.unlock_channel();

    }

    fn select_disk(&self) {
        let ata_channel = unsafe { &*self.from_channel };
        let port_base = ata_channel.port_base;
        
        // device寄存器
        let device_register = CommandBlockRegister::Device(DeviceRegister::new(0, self.primary, true));
        // 写入device寄存器
        pio::write_to_register(port_base, device_register)
    }
    /**
     * 设置要操作的扇区属性
     *  - lba：扇区开始地址
     *  - sector_cnt: 要操作的扇区数量
     */
    fn set_op_sector(&self, lba: u32, sector_cnt: u16) {
        // 得到ATA bus通道
        let ata_channel = unsafe { &*self.from_channel };
        let port_base = ata_channel.port_base;

        // lba地址[0, 8)位
        pio::write_to_register(port_base, CommandBlockRegister::LBALow(lba as u8));
        // lba地址[8, 16)位
        pio::write_to_register(port_base, CommandBlockRegister::LBAMid((lba >> 8) as u8));
        // lba地址[16, 24)位
        pio::write_to_register(port_base, CommandBlockRegister::LBAHigh((lba >> 16) as u8));
        
        // device寄存器。填充：lba地址[24, 28)，是否主盘，是否LBA地址格式
        let device_register = DeviceRegister::new((lba >> 24) as u8, self.primary, true);
        // 把数据真实写入到这个寄存器
        pio::write_to_register(port_base, CommandBlockRegister::Device(device_register));

        // 写入sector count寄存器，设置要操作的扇区数量。如果sec_cnt是256，那么高位截断，就是0
        pio::write_to_register(port_base, CommandBlockRegister::SectorCount(sector_cnt as u8))
    }

    /**
     * 设置要操作的命令，读或者写
     */
    fn set_command(&mut self, command: pio::PIOCommand) {
        // 得到ATA bus通道
        let ata_channel = unsafe { &mut *self.from_channel };
        let port_base = ata_channel.port_base;
        
        // 往命令寄存器写入操作的命令。读或者写
        pio::write_to_register(port_base.try_into().unwrap(), CommandBlockRegister::Command(CommandRegister::new(command)));
        ata_channel.expecting_intr = true;
    }

    /**
     * 检查status寄存器，查看硬盘是否就绪
     */
    fn ready_for_read(&self) -> bool {
        // 得到ATA bus通道
        let ata_channel = unsafe { &*self.from_channel };
        let port_base = ata_channel.port_base;

        let mut status_register = StatusRegister::empty();
        loop {
            let regular_status = CommandBlockRegister::RegularStatus(&mut status_register);
            // 读取status寄存器
            pio::read_from_register(port_base, regular_status);
            // 不忙且可以数据请求
            if !status_register.busy() {
                if  status_register.data_request() {
                    return true;
                }
                return false;
            }
            printk!("status:0b{:b}, not ready", status_register.data);
        }
        return true;
    }

    /**
     * 从该通道的数据寄存器中，读取bytes个字节的数据到buf缓冲区中
     */
    fn read_bytes(&self, buf: &[u8], bytes: u32) {
        // 得到ATA bus通道
        let ata_channel = unsafe { &*self.from_channel };
        let port_base = ata_channel.port_base;

        // 从data寄存器中读取出数据，放到buf地址处的空间中
        pio::read_from_register(port_base, CommandBlockRegister::Data(buf, bytes));
    }

    /**
     * 把buf缓冲区中bytes个字节的数据，写入到该通道的data寄存器中
     */
    fn write_bytes(&mut self, buf: &[u8], bytes: u32) {
        // 得到ATA bus通道
        let ata_channel = unsafe { &*self.from_channel };
        let port_base = ata_channel.port_base;

        // 两种方式；方式一：连续读取字节
        // 把buf缓冲区中的bytes歌字节的数据，写入到硬盘中
        pio::write_to_register(port_base, CommandBlockRegister::Data(buf, bytes));

        // 方式二：
        // // 逐个字（2个字节）读取
        // for start_byte in (0 .. bytes).step_by(2) {
        //     let register = CommandBlockRegister::Data(&buf[start_byte as usize ..], 2);
        //     pio::write_to_register(port_base, register);

        //     // 刷新一下，再写入
        //     self.set_command(PIOCommand::Flush);
        // }
    }
    /**
     * 给该硬盘所属的通道加锁
     */
    fn lock_channel(&mut self) {
        let ata_channel = unsafe { &mut *self.from_channel };
        ata_channel.lock.lock();
    }
    /**
     * 该该硬盘所属的通道解锁
     */
    fn unlock_channel(&mut self) {
        let ata_channel = unsafe { &mut *self.from_channel };
        ata_channel.lock.unlock();
    }

    /**
     * 阻塞该硬盘所属通道的操作，阻塞等待
     */
    fn block(&mut self) {
        let ata_channel = unsafe { &mut *self.from_channel };
        ata_channel.disk_done.down();
    }
    /**
     * 解除该硬盘所属通道的阻塞
     */
    fn unblock(&mut self) {
        let ata_channel = unsafe { &mut *self.from_channel };
        ata_channel.disk_done.up();
    }
}


/**
 * 硬盘中的分区结构（逻辑结构）
 */
#[derive(Debug)]
pub struct Partition {
    /**
     * 分区名称
     * 以C字符串的格式存储
     */
    name: [u8; constants::DISK_NAME_LEN],
    /**
     * 该分区位于硬盘的起始扇区数
     */
    lba_start: LbaAddr,
    /**
     * 该分区占用的扇区数量
     */
    pub sec_cnt: u32,
    /**
     * 该分区归属的硬盘
     */
    pub from_disk: *mut Disk,
    /**
     * 组成链表的tag
     */
    pub tag: LinkedNode,
}


// 自己保证并发问题
unsafe impl Send for Partition {}
unsafe impl Sync for Partition {}


impl Partition {
    pub const fn empty() -> Self {
        Self {
            name: [0; constants::DISK_NAME_LEN],
            lba_start: LbaAddr::empty(),
            sec_cnt: 0,
            from_disk: ptr::null_mut(),
            tag: LinkedNode::new(),
        }
    }

    pub fn new(name: &[u8], lba_start: LbaAddr, sec_cnt: u32, from_disk: *mut Disk) -> Self {
        ASSERT!(name.len() >= constants::DISK_NAME_LEN);
        let mut name_buf = [0; constants::DISK_NAME_LEN];
        name_buf.copy_from_slice(&name[0 .. constants::DISK_NAME_LEN]);
        Self {
            name: name_buf,
            lba_start: lba_start,
            sec_cnt: sec_cnt,
            from_disk: from_disk,
            tag: LinkedNode::new(),
        }
    }

    /**
     * 根据该分区的相对LBA地址，得到绝对LBA地址
     */
    pub fn abs_lba_start(&self, rel_lba_start: u32) -> LbaAddr {
        self.lba_start.add(rel_lba_start)
    }

    pub fn parse_by_tag(tag: *const LinkedNode) -> &'static mut Partition {
        let part = elem2entry!(Partition, tag, tag as *const _ as usize);
        unsafe { &mut *part }
    }

    /**
     * 获取名字
     */
    pub fn get_name(&self) -> &str {
        let name = cstring_utils::read_from_bytes(&self.name);
        ASSERT!(name.is_some());
        name.unwrap()
    }
}


/**
 * 通道起始端口号
 */
pub enum ChannelPortBaseEnum {
    Primary = 0x1f0,
    Secondary = 0x170,
}
/**
 * 通道请求端口号。8259A设定的起始端口号 + 14/15
 *  这里14和15是8259A的这个端口位置。看下面Primary ATA和 Secondary ATA
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
pub enum ChannelIrqNoEnum {
    Primary = (constants::INTERRUPT_NO_START + 14) as isize,
    Secondary = (constants::INTERRUPT_NO_START + 15) as isize,
}



/**
 * 硬盘identify命令得到的内容
 */
#[repr(C, packed)]
pub struct DiskIdentifyContent {
    // 10个字，预留
    reserved1: [u16; 10],
    // sn号，20个字节
    sn: [u8; 20],
    // 7个字，预留
    reserved2: [u16; 7],
    // 硬盘型号。40字节
    module: [u8; 40],
    // 13个字，预留
    reserved3: [u16; 13],
    // 可用扇区数量。4字节
    sec_cnt: u32,
}



