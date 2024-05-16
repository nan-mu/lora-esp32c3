use core::cell::RefCell;

use alloc::{collections::VecDeque, string::String, vec::Vec};
use critical_section::Mutex;
use embedded_graphics::{pixelcolor::Rgb565, prelude::*};

pub static COMMAND_BUCKET: Mutex<RefCell<Option<VecDeque<Command>>>> =
    Mutex::new(RefCell::new(None));
pub static DELAY_BUCKET: Mutex<RefCell<Option<VecDeque<usize>>>> = Mutex::new(RefCell::new(None));

#[derive(Debug)]
pub enum Command {
    Ping,
    /// 前者颜色，后者指定灯的位置，(命令格式：@color,position)
    Blink(Rgb565, Position),
    /// 命令格式：#color,position,time
    DelayBlink(Rgb565, Position, usize),
    /// 重新初始化屏幕
    Reload,
}

#[derive(Debug, Clone)]
pub enum Position {
    Left,
    Right,
    Middle,
}

#[derive(Debug)]
pub enum CommandErr {
    FaillToParse,
    InvalidString,
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
        if value.len() < 1 {
            return Err(CommandErr::InvalidString);
        }
        match &value[..1] {
            "p" => {
                if value == "ping" {
                    Ok(Command::Ping)
                } else {
                    Err(CommandErr::InvalidString)
                }
            }
            "r" => {
                if value == "reload" {
                    Ok(Command::Reload)
                } else {
                    Err(CommandErr::InvalidString)
                }
            }
            "@" => {
                let mut iter = value[1..].split(',');
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
            "#" => {
                let mut iter = value[1..].split(',');
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

                let delay = match iter.next() {
                    Some(delay) => match delay.parse::<u32>() {
                        Ok(delay) => Ok(delay),
                        Err(_) => Err(CommandErr::FaillToParse),
                    },
                    None => Err(CommandErr::FaillToParse),
                }?;
                Ok(Command::DelayBlink(color, position, delay as usize))
            }
            _ => Err(CommandErr::InvalidString),
        }
    }
}
