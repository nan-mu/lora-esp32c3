//! This shows how to configure UART
//! You can short the TX and RX pin and see it reads what was written.
//! Additionally you can connect a logic analzyer to TX and see how the changes
//! of the configuration change the output signal.
//!
//! The following wiring is assumed:
//! - TX => GPIO4
//! - RX => GPIO5

//% CHIPS: esp32 esp32c2 esp32c3 esp32c6 esp32h2 esp32s2 esp32s3

#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    // delay::Delay,
    gpio::IO,
    peripherals::Peripherals,
    prelude::*,
    uart::{
        config::{Config, DataBits, Parity, StopBits},
        ClockSource, TxRxPins, Uart,
    },
};
use esp_println::{print, println};
use nb::block;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

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

    // let delay = Delay::new(&clocks);

    println!("Start");
    loop {
        let read = block!(serial1.read_byte());
        match read {
            Ok(read) => print!("{}", read as char),
            Err(err) => println!("Error {:?}", err),
        }
        // delay.delay_millis(10);
    }
}
