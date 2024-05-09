use crate::{port::Port, utils};

/**
 * 可编程计数器 programmable interval timers 
 * <https://en.wikipedia.org/wiki/Intel_8253#Features>
 * 可编程计数器的物理结构分为两部分，对应的端口分别是：
 *  - 控制器寄存器
 *      - 0x43
 *  - 计数器
 *      - 计数器0： 0x40
 *      - 计数器1： 0x41
 *      - 计数器2： 0x42
 */

/**
 * 8253控制字的结构
 */
pub struct ControlWord {
    data:u8
}
impl ControlWord {
    pub fn new(bcd: bool, op_mode: OperationMode, rw_format: ReadWriteFormat, counter_no: CounterNo) -> Self {
        Self {
            data: 
                utils::bool_to_u8(bcd) | 
                (op_mode as u8) << 1 |
                (rw_format as u8) << 4 |
                (counter_no as u8) << 6
        }
    }
}

pub fn init_pit() {
    let cw = ControlWord::new(true, OperationMode::RateGenerator,ReadWriteFormat::LowThenHigh, CounterNo::Counter0);
    // 写入控制器
    Port::<u8>::new(0x43).write(cw.data);
    
    
    Port::<u8>::new(0x40).write()

}

/**
 * 工作模式
 * <https://en.wikipedia.org/wiki/Intel_8253#Operation_modes>
 */
pub enum OperationMode {
    InterruptOnTerminalCount = 0x0,
    ProgrammableOneShot, 
    RateGenerator,
    SquareWaveGenerator,
    SoftwareTriggeredStrobe,
    HardwareTriggeredStrobe,
}
pub enum ReadWriteFormat {
    /**
     * 锁存数据
     */
    LatchCountValue = 0x0,
    /**
     * 只读低字节
     */
    LowByteOnly,
    /**
     * 只读高字节
     */
    HighByteOnly,
    /**
     * 先读低再读高
     */
    LowThenHigh,
}

/**
 * 选择的计数器编号
 */
pub enum CounterNo {
    Counter0 = 0x0,
    Counter1,
    Counter2,
    Undefined
}

