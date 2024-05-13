use alloc::{string::String, vec::Vec};
use embedded_graphics::{pixelcolor::Rgb565, prelude::*};

pub enum Command {
    Ping,
    /// 前者颜色，后者指定灯的位置，(命令格式：@color,position)
    Blink(Rgb565, Position),
}

pub enum Position {
    Left,
    Right,
    Middle,
}

impl From<&str> for Command {
    fn from(value: &str) -> Self {
        let mut iter = value.split(',');
        let color = iter.next().unwrap();
        let position = iter.next().unwrap();
        let color = match color {
            "red" => Rgb565::RED,
            "green" => Rgb565::GREEN,
            "blue" => Rgb565::BLUE,
            _ => Rgb565::BLACK,
        };
        let position = match position {
            "left" => Position::Left,
            "right" => Position::Right,
            "middle" => Position::Middle,
            _ => Position::Middle,
        };
        Command::Blink(color, position)
    }
}

pub fn match_command(buf: &Vec<char>) -> Option<Command> {
    let buf = buf.iter().collect::<String>();
    match buf.as_str() {
        "ping" => Some(Command::Ping),
        _ => None,
    }
}
