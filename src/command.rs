use alloc::{string::String, vec::Vec};

pub enum Command {
    Ping,
}

pub fn match_command(buf: &Vec<char>) -> Option<Command> {
    let buf = buf.iter().collect::<String>();
    match buf.as_str() {
        "ping" => Some(Command::Ping),
        _ => None,
    }
}
