use crate::{constants, port::Port, utils, ASSERT};

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
const COUNTER_ENUM: CounterEnum = CounterEnum::new();


pub struct ControlWord {
    data: u8,
}
impl ControlWord {
    pub fn new(
        bcd: bool,
        op_mode: OperationMode,
        rw_format: ReadWriteFormat,
        counter_no: u8,
    ) -> Self {
        Self {
            data: utils::bool_to_u8(bcd)
                | (op_mode as u8) << 1
                | (rw_format as u8) << 4
                | counter_no << 6,
        }
    }
}

/**
 * 时钟中断频率初始化
 */
pub fn pit_init() {
    // 设置counter0，的频率为100次/s
    set_frequency(&COUNTER_ENUM.counter0, constants::TIMER_INTR_FREQUENCY);
}
/**
 * 设置中断频率
 *  - counter: 要设置的计数器
 *  - intr_frequency: 希望发出中断的频率。intr_frequency次/秒
 */
pub fn set_frequency(counter: &'static dyn CounterTrait, intr_frequency: u16) {
    // 往计数器赋值。抵脉冲信号
    let value = constants::PIT_DEFAULT_FREQUENCY / intr_frequency as u32;
    ASSERT!(value <= constants::PIC_MAX_DECREMENT);
    let value = value as u16;
    // 构建控制字
    let cw = ControlWord::new(
        false,
        OperationMode::RateGenerator,
        ReadWriteFormat::LowThenHigh,
        counter.get_no(),
    );
    // 写入控制器
    Port::<u8>::new(0x43).write(cw.data);
    let counter0_port = Port::<u8>::new(counter.get_port());
    // 先写低8位
    counter0_port.write(value as u8);
    // 在写高8位
    counter0_port.write((value >> 8) as u8);
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
    Undefined,
}

struct CounterEnum {
    counter0: Counter,
    counter1: Counter,
    counter2: Counter,
}
impl CounterEnum {
    pub const fn new() -> CounterEnum {
        Self {
            counter0: Counter {
                no: 0x0,
                port: 0x40,
            },
            counter1: Counter {
                no: 0x1,
                port: 0x41,
            },
            counter2: Counter {
                no: 0x2,
                port: 0x42,
            },
        }
    }
}
trait CounterTrait {
    fn get_no(&self) -> u8;
    fn get_port(&self) -> u16;
}
struct Counter {
    no: u8,
    port: u16,
}
impl CounterTrait for Counter {
    fn get_no(&self) -> u8 {
        self.no
    }

    fn get_port(&self) -> u16 {
        self.port
    }
}
