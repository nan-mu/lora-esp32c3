use crate::time::UpdateIndex;
use crate::{command::Command, time::NOW};
use alloc::borrow::ToOwned;
use core::mem::MaybeUninit;
use embedded_graphics::{
    mono_font::{
        ascii::{FONT_6X10, FONT_7X14},
        MonoTextStyle,
    },
    pixelcolor::Rgb565,
    prelude::*,
    primitives::Circle,
    primitives::{PrimitiveStyle, PrimitiveStyleBuilder},
    text::Text,
};
use embedded_hal::digital::OutputPin;
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::{
    delay::Delay,
    gpio, peripherals,
    spi::{self, master::Spi},
};
use esp_println::println;
use st7735_lcd::ST7735;

// 字体与颜色
pub const TEXT_COLOR: Rgb565 = Rgb565::WHITE;
pub const BG_COLOR: Rgb565 = Rgb565::BLACK;
pub static STYLE: MonoTextStyle<'_, Rgb565> = MonoTextStyle::new(&FONT_7X14, TEXT_COLOR);
pub static NUM: [&str; 10] = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"];

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

static mut TIME_TEXT: (
    (
        Text<'_, MonoTextStyle<'_, Rgb565>>,
        Text<'_, MonoTextStyle<'_, Rgb565>>,
    ),
    (
        Text<'_, MonoTextStyle<'_, Rgb565>>,
        Text<'_, MonoTextStyle<'_, Rgb565>>,
    ),
    (
        Text<'_, MonoTextStyle<'_, Rgb565>>,
        Text<'_, MonoTextStyle<'_, Rgb565>>,
    ),
) = (
    (
        Text::new(NUM[0], Point::new(40, 60), STYLE),
        Text::new(NUM[0], Point::new(50, 60), STYLE),
    ),
    (
        Text::new(NUM[0], Point::new(60, 60), STYLE),
        Text::new(NUM[0], Point::new(70, 60), STYLE),
    ),
    (
        Text::new(NUM[0], Point::new(80, 60), STYLE),
        Text::new(NUM[0], Point::new(90, 60), STYLE),
    ),
);

const RIGHT_X: u16 = 159;
const BOTTOM_Y: u16 = 79;

pub fn 屏幕初始化<SPI, DC, RST>(device: &mut ST7735<SPI, DC, RST>, delay: &mut Delay)
where
    SPI: embedded_hal::spi::SpiDevice,
    DC: OutputPin,
    RST: OutputPin,
{
    device.init(delay).unwrap();
    device.clear(BG_COLOR).unwrap();
    device
        .set_orientation(&st7735_lcd::Orientation::Landscape)
        .unwrap(); //表示屏幕竖着来（反向）
    device.set_offset(1, 26);
}

pub fn 绘制边框<SPI, DC, RST>(device: &mut ST7735<SPI, DC, RST>)
where
    SPI: embedded_hal::spi::SpiDevice,
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

pub fn 出问题了(text: &str) {
    unsafe {
        let device = &mut *ST7735.as_mut_ptr();
        let style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
        Text::new(text, Point::new(20, 30), style)
            .draw(device)
            .unwrap();
    }
}

pub unsafe fn 绘制数字<SPI, DC, RST>(device: &mut ST7735<SPI, DC, RST>)
where
    SPI: embedded_hal::spi::SpiDevice,
    DC: OutputPin,
    RST: OutputPin,
{
    TIME_TEXT.0 .0.text = NUM[NOW.hour.0 as usize];
    TIME_TEXT.0 .0.draw(device).unwrap();

    TIME_TEXT.0 .1.text = NUM[NOW.hour.1 as usize];
    TIME_TEXT.0 .1.draw(device).unwrap();

    TIME_TEXT.1 .0.text = NUM[NOW.min.0 as usize];
    TIME_TEXT.1 .0.draw(device).unwrap();

    TIME_TEXT.1 .1.text = NUM[NOW.min.1 as usize];
    TIME_TEXT.1 .1.draw(device).unwrap();

    TIME_TEXT.2 .0.text = NUM[NOW.sec.0 as usize];
    TIME_TEXT.2 .0.draw(device).unwrap();

    TIME_TEXT.2 .1.text = NUM[NOW.sec.1 as usize];
    TIME_TEXT.2 .1.draw(device).unwrap();
}

pub fn 更新时间() {
    unsafe {
        log::info!("时钟中断：{}", NOW);
        for index in NOW.add_sec().iter() {
            match &index {
                UpdateIndex::Hour10 => {
                    TIME_TEXT
                        .0
                         .0
                        .bounding_box()
                        .into_styled(PrimitiveStyle::with_fill(BG_COLOR))
                        .draw(&mut *ST7735.as_mut_ptr())
                        .unwrap();
                    TIME_TEXT.0 .0.text = NUM[NOW.hour.0 as usize];
                    TIME_TEXT.0 .0.draw(&mut *ST7735.as_mut_ptr()).unwrap();
                }
                UpdateIndex::Hour1 => {
                    TIME_TEXT
                        .0
                         .1
                        .bounding_box()
                        .into_styled(PrimitiveStyle::with_fill(BG_COLOR))
                        .draw(&mut *ST7735.as_mut_ptr())
                        .unwrap();
                    TIME_TEXT.0 .1.text = NUM[NOW.hour.1 as usize];
                    TIME_TEXT.0 .1.draw(&mut *ST7735.as_mut_ptr()).unwrap();
                }
                UpdateIndex::Min10 => {
                    TIME_TEXT
                        .1
                         .0
                        .bounding_box()
                        .into_styled(PrimitiveStyle::with_fill(BG_COLOR))
                        .draw(&mut *ST7735.as_mut_ptr())
                        .unwrap();
                    TIME_TEXT.1 .0.text = NUM[NOW.min.0 as usize];
                    TIME_TEXT.1 .0.draw(&mut *ST7735.as_mut_ptr()).unwrap();
                }
                UpdateIndex::Min1 => {
                    TIME_TEXT
                        .1
                         .1
                        .bounding_box()
                        .into_styled(PrimitiveStyle::with_fill(BG_COLOR))
                        .draw(&mut *ST7735.as_mut_ptr())
                        .unwrap();
                    TIME_TEXT.1 .1.text = NUM[NOW.min.1 as usize];
                    TIME_TEXT.1 .1.draw(&mut *ST7735.as_mut_ptr()).unwrap();
                }
                UpdateIndex::Sec10 => {
                    TIME_TEXT
                        .2
                         .0
                        .bounding_box()
                        .into_styled(PrimitiveStyle::with_fill(BG_COLOR))
                        .draw(&mut *ST7735.as_mut_ptr())
                        .unwrap();
                    TIME_TEXT.2 .0.text = NUM[NOW.sec.0 as usize];
                    TIME_TEXT.2 .0.draw(&mut *ST7735.as_mut_ptr()).unwrap();
                }
                UpdateIndex::Sec1 => {
                    TIME_TEXT
                        .2
                         .1
                        .bounding_box()
                        .into_styled(PrimitiveStyle::with_fill(BG_COLOR))
                        .draw(&mut *ST7735.as_mut_ptr())
                        .unwrap();
                    TIME_TEXT.2 .1.text = NUM[NOW.sec.1 as usize];
                    TIME_TEXT.2 .1.draw(&mut *ST7735.as_mut_ptr()).unwrap();
                }
            }
        }
    }
}
