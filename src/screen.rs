use alloc::borrow::ToOwned;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::PrimitiveStyleBuilder;
use embedded_graphics::{pixelcolor::Rgb565, primitives::Circle};
use embedded_hal::{digital::OutputPin, spi};
use esp_hal::delay::Delay;
use esp_println::println;
use st7735_lcd::ST7735;

use crate::command::Command;

const RIGHT_X: u16 = 159;
const BOTTOM_Y: u16 = 79;

// 字体与颜色
// const TEXT_COLOR: Rgb565 = Rgb565::WHITE;
pub const BG_COLOR: Rgb565 = Rgb565::BLACK;

pub fn 屏幕初始化<SPI, DC, RST>(device: &mut ST7735<SPI, DC, RST>, delay: &mut Delay)
where
    SPI: spi::SpiDevice,
    DC: OutputPin,
    RST: OutputPin,
{
    device.init(delay).unwrap();
    device.clear(BG_COLOR).unwrap();
    device
        .set_orientation(&st7735_lcd::Orientation::LandscapeSwapped)
        .unwrap(); //表示屏幕竖着来（反向）
    device.set_offset(1, 26);
}

pub fn 绘制边框<SPI, DC, RST>(device: &mut ST7735<SPI, DC, RST>)
where
    SPI: spi::SpiDevice,
    DC: OutputPin,
    RST: OutputPin,
{
    for x in 0..=RIGHT_X {
        // 屏幕下边线
        device.set_pixel(x, BOTTOM_Y, 0xffff).unwrap();
    }

    for y in 0..=BOTTOM_Y {
        // 屏幕右边线
        device.set_pixel(RIGHT_X, y, 0xffff).unwrap();
    }

    for x in 0..=RIGHT_X {
        // 屏幕上边线
        device.set_pixel(x, 0, 0xffff).unwrap();
    }

    for y in 0..=BOTTOM_Y {
        // 屏幕左边线
        device.set_pixel(0, y, 0xffff).unwrap();
    }
}

pub fn 改变灯的颜色(command: &Command) {
    unsafe {
        use crate::ST7735;
        let device = &mut *ST7735.as_mut_ptr();
        if let Command::Blink(color, position) = command {
            let circle = Circle::with_center(
                Point::new(
                    match position {
                        crate::command::Position::Left => (RIGHT_X / 4) as i32,
                        crate::command::Position::Middle => (RIGHT_X / 2) as i32,
                        crate::command::Position::Right => (RIGHT_X / 4 * 3) as i32,
                    },
                    (BOTTOM_Y / 2) as i32,
                ),
                20,
            );
            let style = PrimitiveStyleBuilder::new()
                .fill_color(color.to_owned())
                .build();
            println!("{:?}", circle);
            circle.into_styled(style).draw(device).unwrap();
        } else {
            panic!("Unknown command")
        }
    }
}
