use alloc::{string::String, vec::Vec};
use embedded_graphics::{pixelcolor::Rgb565, prelude::*};

#[derive(Debug)]
pub enum Command {
    Ping,
    /// 前者颜色，后者指定灯的位置，(命令格式：@color,position)
    Blink(Rgb565, Position),
}

#[derive(Debug)]
pub enum Position {
    Left,
    Right,
    Middle,
}

#[derive(Debug)]
pub enum CommandErr {
    FaillToParse,
}

impl TryFrom<&Vec<char>> for Command {
    type Error = CommandErr;
    fn try_from(value: &Vec<char>) -> Result<Self, Self::Error> {
        let value: String = value.iter().collect();
        Command::try_from(value.as_str())
    }
}

impl TryFrom<&str> for Command {
    type Error = CommandErr;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "ping" => Ok(Command::Ping),
            _ => {
                let mut iter = value.split(',');
                let color = match iter.next() {
                    Some(color) => match color {
                        "red" => Ok(Rgb565::RED),
                        "green" => Ok(Rgb565::GREEN),
                        "blue" => Ok(Rgb565::BLUE),
                        _ => Err(CommandErr::FaillToParse),
                    },
                    None => Err(CommandErr::FaillToParse),
                }?;

                let position = match iter.next() {
                    Some(position) => match position {
                        "left" => Ok(Position::Left),
                        "right" => Ok(Position::Right),
                        "middle" => Ok(Position::Middle),
                        _ => Err(CommandErr::FaillToParse),
                    },
                    None => Err(CommandErr::FaillToParse),
                }?;
                Ok(Command::Blink(color, position))
            }
        }
    }
}
