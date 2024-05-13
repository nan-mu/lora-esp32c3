#![no_std]
#![no_main]

mod command;
use command::{match_command, Command};

extern crate alloc;
use alloc::vec::Vec;
use core::mem::MaybeUninit;

use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    // delay::Delay,
    gpio::IO,
    peripherals::Peripherals,
    prelude::{nb::block, *},
    uart::{
        config::{Config, DataBits, Parity, StopBits},
        ClockSource, TxRxPins, Uart,
    },
};
use esp_println::{print, println};

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
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    init_heap();

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

    println!("Start");
    let mut buf = Vec::new();
    loop {
        // 这里遇到了一些问题，hal库中有read_byte()和drain_fifo()两个方法从串口读取数据，前者一个字符一个字符读，后者一次性读取所有数据，而后者无法正常使用，所以还是使用比较原始的方法读取
        match block!(serial1.read_byte()) {
            Ok(byte) => match byte {
                b'\n' => {
                    match match_command(&buf) {
                        Some(Command::Ping) => {
                            serial1.write_bytes(b"pong").unwrap();
                            println!("pong");
                        }
                        None => {
                            println!("Unknown command");
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
