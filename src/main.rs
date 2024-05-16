#![no_std]
#![no_main]

mod command;
mod screen;
mod time;
use command::Command;

extern crate alloc;
use alloc::{borrow::ToOwned, vec::Vec};
use core::{cell::RefCell, mem::MaybeUninit};
use critical_section::Mutex;
use embedded_hal_bus::spi::ExclusiveDevice;

use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    delay::Delay,
    gpio::{self, IO},
    interrupt::{self, Priority},
    peripherals::{self, Interrupt, Peripherals, TIMG0},
    prelude::{nb::block, *},
    spi::{self, master::Spi},
    timer::{Timer, Timer0, TimerGroup, TimerInterrupts},
    uart::{
        config::{Config, DataBits, Parity, StopBits},
        ClockSource, TxRxPins, Uart,
    },
};
use esp_println::{print, println};

use crate::{screen::改变灯的颜色, time::tg0_t0_level};

#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

fn init_heap() {
    const HEAP_SIZE: usize = 32 * 1024;
    static mut HEAP: MaybeUninit<[u8; HEAP_SIZE]> = MaybeUninit::uninit();

    unsafe {
        ALLOCATOR.init(HEAP.as_mut_ptr() as *mut u8, HEAP_SIZE);
    }
}

pub static mut ST7735: MaybeUninit<
    st7735_lcd::ST7735<
        ExclusiveDevice<
            Spi<'static, peripherals::SPI2, spi::FullDuplexMode>,
            gpio::GpioPin<gpio::Output<gpio::PushPull>, 7>,
            Delay,
        >,
        gpio::GpioPin<gpio::Output<gpio::PushPull>, 6>,
        gpio::GpioPin<gpio::Output<gpio::PushPull>, 10>,
    >,
> = MaybeUninit::uninit();

pub static TIMER0: Mutex<RefCell<Option<Timer<Timer0<TIMG0>, esp_hal::Blocking>>>> =
    Mutex::new(RefCell::new(None));

pub static HANDLER_COMMAND: Mutex<RefCell<Option<Vec<(Command, usize)>>>> =
    Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    init_heap();
    let mut delay = Delay::new(&clocks);

    // 初始化时钟中断
    let timg0 = TimerGroup::new(
        peripherals.TIMG0,
        &clocks,
        Some(TimerInterrupts {
            timer0_t0: Some(tg0_t0_level),
            ..Default::default()
        }),
    );
    let timer0 = timg0.timer0;

    interrupt::enable(Interrupt::TG0_T0_LEVEL, Priority::Priority1).unwrap();
    // timer0.start(1000u64.millis());
    // timer0.listen();

    critical_section::with(|cs| {
        TIMER0.borrow_ref_mut(cs).replace(timer0);
    });

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
        use crate::screen::{屏幕初始化, 绘制边框};
        ST7735
            .as_mut_ptr()
            .write(st7735_lcd::ST7735::new(spi, dc, res, false, true, 110, 161));
        屏幕初始化(&mut *ST7735.as_mut_ptr(), &mut delay);
        绘制边框(&mut *ST7735.as_mut_ptr());
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
                        Ok(Command::Blink(color, position)) => {
                            println!("Blink {:?} {:?}", color, position);
                            改变灯的颜色(&command.unwrap());
                        }
                        Ok(Command::DelayBlink(color, position, delay)) => {
                            println!("DelayBlink {:?} {:?} {:?}", color, position, delay);
                            critical_section::with(|cs| {
                                let mut command_bucket = HANDLER_COMMAND.borrow_ref_mut(cs);
                                let command_bucket = command_bucket.as_mut().unwrap();
                                command_bucket.push((
                                    Command::Blink(color.to_owned(), position.to_owned()),
                                    *delay,
                                ));
                            });
                        }
                        Err(e) => {
                            println!("Unknown command {:?}", e);
                        }
                    }
                    buf.clear();
                }
                b'\r' => {
                    buf.clear();
                }
                _ => {
                    print!("{}", byte as char);
                    buf.push(byte as char);
                }
            },
            Err(_) => continue,
        }
    }
}
