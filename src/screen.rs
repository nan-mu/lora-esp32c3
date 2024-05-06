use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_hal::{digital::OutputPin, spi};
use esp_hal::delay::Delay;
use st7735_lcd::ST7735;

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
    let (right_x, bottom_y) = (159, 79);

    for x in 0..=right_x {
        // 屏幕下边线
        device.set_pixel(x, bottom_y, 0xffff).unwrap();
    }

    for y in 0..=bottom_y {
        // 屏幕右边线
        device.set_pixel(right_x, y, 0xffff).unwrap();
    }

    for x in 0..=right_x {
        // 屏幕上边线
        device.set_pixel(x, 0, 0xffff).unwrap();
    }

    for y in 0..=bottom_y {
        // 屏幕左边线
        device.set_pixel(0, y, 0xffff).unwrap();
    }
}
