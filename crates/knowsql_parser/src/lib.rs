use nom::{branch::alt, IResult};

pub mod command;
pub mod resp2;
pub mod simple;

pub mod protocol;

/// Parse a command from a string. Attempt to parse as RESP2 first, then fallback to simple.
pub fn parse_command(input: &str) -> IResult<&str, command::Command> {
    alt((simple::parse_command, resp2::parse_command))(input)
}

pub fn try_parse_command(input: &str) -> Option<command::Command> {
    match parse_command(input) {
        Ok((_, command)) => Some(command),
        Err(_) => None,
    }
}
