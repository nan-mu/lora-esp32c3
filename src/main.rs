#![no_std]
#![no_main]

mod command;
mod screen;
mod time;

extern crate alloc;

use alloc::{borrow::ToOwned, collections::VecDeque, format, string::ToString, vec::Vec};
use command::Command;
use core::mem::MaybeUninit;
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    delay::Delay,
    gpio::{self, IO},
    interrupt::{self, Priority},
    peripherals::{Interrupt, Peripherals},
    prelude::{nb::block, *},
    spi::master::Spi,
    systimer::SystemTimer,
    timer::{TimerGroup, TimerInterrupts},
    uart::{
        config::{Config, DataBits, Parity, StopBits},
        ClockSource, TxRxPins, Uart,
    },
};
use esp_println::println;
use fugit::ExtU32;

#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

fn init_heap() {
    const HEAP_SIZE: usize = 32 * 1024;
    static mut HEAP: MaybeUninit<[u8; HEAP_SIZE]> = MaybeUninit::uninit();

    unsafe {
        ALLOCATOR.init(HEAP.as_mut_ptr() as *mut u8, HEAP_SIZE);
    }
}

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let systimer = SystemTimer::new(peripherals.SYSTIMER);
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    let mut delay = Delay::new(&clocks);
    init_heap();

    // 初始化时钟
    let timg0 = TimerGroup::new(
        peripherals.TIMG0,
        &clocks,
        Some(TimerInterrupts {
            timer0_t0: Some(time::tg0_t0_level),
            ..Default::default()
        }),
    );
    let mut timer0 = timg0.timer0;
    timer0.start(1000u64.millis());
    timer0.listen();

    // 初始化系统时间闹钟
    let mut alarm0 = systimer.alarm0.into_periodic();
    alarm0.set_interrupt_handler(time::systimer_target0);

    critical_section::with(|cs| {
        time::TIMER0.borrow_ref_mut(cs).replace(timer0);

        // 顺便初始化用于延时命令的两个栈
        command::COMMAND_BUCKET
            .borrow_ref_mut(cs)
            .replace(VecDeque::new());
        command::DELAY_BUCKET
            .borrow_ref_mut(cs)
            .replace(VecDeque::new());

        // 初始化系统时间的闹钟
        time::ALARM0.borrow_ref_mut(cs).replace(alarm0);
    });

    interrupt::enable(Interrupt::SYSTIMER_TARGET0, Priority::Priority1).unwrap();
    interrupt::enable(Interrupt::TG0_T0_LEVEL, Priority::Priority1).unwrap();

    // 初始化时间
    // 加载时间必须要在刷新屏幕之前，屏幕刷新太耗时了
    // 时间格式1996-12-19T16:39:57-08:00
    let now: &[u8] = include_bytes!("../assets/time.bin");
    log::info!("解析时间： {:?}", now);
    unsafe {
        time::NOW.build(now);
        log::info!("运行时获得时间： {}", time::NOW);
    }

    // 初始化串口设备
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let pins = TxRxPins::new_tx_rx(
        io.pins.gpio0.into_push_pull_output(),
        io.pins.gpio1.into_floating_input(),
    );

    let mut serial1 = Uart::new_with_config(
        peripherals.UART1,
        Config {
            baudrate: 9600,
            data_bits: DataBits::DataBits8,
            clock_source: ClockSource::Xtal,
            parity: Parity::ParityNone,
            stop_bits: StopBits::STOP1,
        },
        Some(pins),
        &clocks,
        None,
    );

    // 初始化屏幕
    // SCK->2 SDA->3 RES->10 DC->6 CS->7
    let sck = io.pins.gpio2;
    let sda = io.pins.gpio3;
    let res = io.pins.gpio10.into_push_pull_output();
    let dc = io.pins.gpio6.into_push_pull_output();
    let cs = io.pins.gpio7.into_push_pull_output();
    let spi = Spi::new(
        peripherals.SPI2,
        1000u32.kHz(),
        esp_hal::spi::SpiMode::Mode0,
        &clocks,
    )
    .with_pins(Some(sck), Some(sda), gpio::NO_PIN, gpio::NO_PIN);
    let spi = ExclusiveDevice::new(spi, cs, delay).unwrap();

    // 其实这里不使用critical_section而是直接unsafe是因为懒得改了，实际不应该这样
    unsafe {
        use screen::{屏幕初始化, 绘制数字, 绘制边框, ST7735};
        ST7735
            .as_mut_ptr()
            .write(st7735_lcd::ST7735::new(spi, dc, res, false, true, 110, 161));
        屏幕初始化(&mut *ST7735.as_mut_ptr(), &mut delay);
        绘制边框(&mut *ST7735.as_mut_ptr());
        绘制数字(&mut *ST7735.as_mut_ptr());
    }

    println!("drew down");

    println!("Start");
    let mut buf = Vec::new();
    loop {
        // 这里遇到了一些问题，hal库中有read_byte()和drain_fifo()两个方法从串口读取数据，前者一个字符一个字符读，后者一次性读取所有数据，而后者无法正常使用，所以还是使用比较原始的方法读取
        match block!(serial1.read_byte()) {
            Ok(byte) => match byte {
                b'\n' => {
                    let command = Command::try_from(&buf);
                    match &command {
                        Ok(Command::Ping) => {
                            serial1.write_bytes(b"pong").unwrap();
                            println!("pong");
                        }
                        Ok(Command::Reload) => unsafe {
                            use screen::{屏幕初始化, 绘制边框, ST7735};
                            屏幕初始化(&mut *ST7735.as_mut_ptr(), &mut delay);
                            绘制边框(&mut *ST7735.as_mut_ptr());
                        },
                        Ok(Command::Blink(color, position)) => {
                            println!("Blink {:?} {:?}", color, position);
                            screen::改变灯的颜色(&command.unwrap());
                        }
                        Ok(Command::DelayBlink(color, position, delay)) => {
                            println!("DelayBlink {:?} {:?} {:?}", color, position, delay);
                            critical_section::with(|cs| {
                                command::COMMAND_BUCKET
                                    .borrow_ref_mut(cs)
                                    .as_mut()
                                    .unwrap()
                                    .push_back(Command::Blink(
                                        color.to_owned(),
                                        position.to_owned(),
                                    ));

                                let mut delay_bucket = command::DELAY_BUCKET.borrow_ref_mut(cs);
                                let delay_bucket = delay_bucket.as_mut().unwrap();
                                delay_bucket.push_back(delay.clone());

                                let mut alarm0 = time::ALARM0.borrow_ref_mut(cs);
                                let alarm0 = alarm0.as_mut().unwrap();

                                if delay_bucket.len() == 1 {
                                    println!("计时器启动");
                                    alarm0.set_period(
                                        (delay_bucket.front().unwrap().clone() as u32).secs(),
                                    );
                                    alarm0.enable_interrupt(true);
                                }
                            });
                        }
                        Err(e) => {
                            serial1
                                .write_bytes(
                                    &format!("Unknown command {:?}", e).to_string().into_bytes(),
                                )
                                .unwrap();
                            screen::出问题了(&format!("Unknown command {:?}", e).to_string());
                            println!("Unknown command {:?}", e);
                        }
                    }
                    buf.clear();
                }
                b'\r' => {
                    buf.clear();
                }
                _ => {
                    buf.push(byte as char);
                }
            },
            Err(_) => continue,
        }
    }
}
