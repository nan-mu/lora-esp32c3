#![no_std]
#![no_main]

extern crate alloc;
use core::{
    cell::{Cell, RefCell},
    mem::MaybeUninit,
};

use critical_section::Mutex;
use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    delay::Delay,
    gpio::IO,
    peripherals::{Peripherals, UART1},
    prelude::*,
    // spi::{master::Spi, FullDuplexMode},
    uart::{
        config::{AtCmdConfig, Config, DataBits, Parity, StopBits},
        ClockSource, TxRxPins, Uart,
    },
    Blocking,
};

// mod screen;

#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

fn init_heap() {
    const HEAP_SIZE: usize = 32 * 1024;
    static mut HEAP: MaybeUninit<[u8; HEAP_SIZE]> = MaybeUninit::uninit();

    unsafe {
        ALLOCATOR.init(HEAP.as_mut_ptr() as *mut u8, HEAP_SIZE);
    }
}

// 屏幕外设控制
// use embedded_hal_bus::spi::ExclusiveDevice;
use esp_println::{print, println};
// static mut ST7735: MaybeUninit<
//     st7735_lcd::ST7735<
//         ExclusiveDevice<
//             Spi<'static, peripherals::SPI2, FullDuplexMode>,
//             GpioPin<Output<PushPull>, 7>,
//             Delay,
//         >,
//         GpioPin<Output<PushPull>, 6>,
//         GpioPin<Output<PushPull>, 10>,
//     >,
// > = MaybeUninit::uninit();

static SERIAL: Mutex<RefCell<Option<Uart<UART1, Blocking>>>> = Mutex::new(RefCell::new(None));
static SIGNAL: Mutex<Cell<bool>> = Mutex::new(Cell::new(false));

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let clocks = ClockControl::max(system.clock_control).freeze();
    let delay = Delay::new(&clocks);
    init_heap();

    esp_println::logger::init_logger_from_env();

    // 屏幕外设初始化
    // SCK->2 SDA->3 RES->10 DC->6 CS->7
    // io.set_interrupt_handler(io_interrupt);
    // let sck = io.pins.gpio2;
    // let sda = io.pins.gpio3;
    // let res = io.pins.gpio10.into_push_pull_output();
    // let dc = io.pins.gpio6.into_push_pull_output();
    // let cs = io.pins.gpio7.into_push_pull_output();
    // let spi = esp_hal::spi::master::Spi::new(
    //     peripherals.SPI2,
    //     1000u32.kHz(),
    //     esp_hal::spi::SpiMode::Mode0,
    //     &clocks,
    // )
    // .with_pins(Some(sck), Some(sda), gpio::NO_PIN, gpio::NO_PIN);
    // 串口外设初始化
    let pins = TxRxPins::new_tx_rx(
        io.pins.gpio0.into_push_pull_output(),
        io.pins.gpio1.into_floating_input(),
    );
    // let spi = embedded_hal_bus::spi::ExclusiveDevice::new(spi, cs, delay).unwrap();

    // unsafe {
    //     use crate::screen::{屏幕初始化, 绘制边框};
    //     ST7735
    //         .as_mut_ptr()
    //         .write(st7735_lcd::ST7735::new(spi, dc, res, false, true, 110, 161));
    //     屏幕初始化(&mut *ST7735.as_mut_ptr(), &mut delay);
    //     // 绘制边框(&mut *ST7735.as_mut_ptr());
    // }

    let mut uart1 = Uart::new_with_config(
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
        Some(interrupt_handler),
    );

    critical_section::with(|cs| {
        uart1.set_at_cmd(AtCmdConfig::new(None, None, None, b'#', None));
        uart1.set_rx_fifo_full_threshold(30).unwrap();
        uart1.listen_at_cmd();
        uart1.listen_rx_fifo_full();

        SERIAL.borrow_ref_mut(cs).replace(uart1);
    });

    println!("初始化结束");

    loop {}
}

#[handler]
fn interrupt_handler() {
    critical_section::with(|cs| {
        println!("catch interrupt");
        let mut serial = SERIAL.borrow_ref_mut(cs);
        let serial = serial.as_mut().unwrap();
        SIGNAL.borrow(cs).set(true);
        // let mut buf = Vec::new();
        while let nb::Result::Ok(c) = serial.read_byte() {
            print!("{}", c as char);
        }
        println!("");

        serial.reset_at_cmd_interrupt();
        serial.reset_rx_fifo_full_interrupt();
    });
}
