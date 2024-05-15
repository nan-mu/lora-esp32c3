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
    InvalidCommand,
    FaillToParse,
}

impl TryFrom<&str> for Command {
    type Error = CommandErr;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
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

pub fn match_command(buf: &Vec<char>) -> Result<Command, CommandErr> {
    let buf = buf.iter().collect::<String>();
    match buf.as_str() {
        "ping" => Ok(Command::Ping),
        _ => Err(CommandErr::InvalidCommand),
    }
}
