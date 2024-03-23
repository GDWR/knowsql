use crate::command::Command;

use nom::{
    branch::alt,
    bytes::streaming::{tag, tag_no_case},
    character::streaming::alphanumeric1,
    IResult,
};

fn parse_db_size(input: &[u8]) -> IResult<&[u8], Command> {
    let (input, _) = tag_no_case("dbsize")(input)?;
    Ok((input, Command::DbSize))
}

fn parse_get(input: &[u8]) -> IResult<&[u8], Command> {
    let (input, _) = tag_no_case("get")(input)?;
    let (input, _) = tag(" ")(input)?;
    let (input, key) = alphanumeric1(input)?;
    Ok((input, Command::Get(std::str::from_utf8(key).expect("key is valid utf8 string"))))
}

fn parse_set(input: &[u8]) -> IResult<&[u8], Command> {
    let (input, _) = tag_no_case("set")(input)?;
    let (input, _) = tag(" ")(input)?;
    let (input, key) = alphanumeric1(input)?;
    let (input, _) = tag(" ")(input)?;
    let (input, value) = alphanumeric1(input)?;
    Ok((input, Command::Set(std::str::from_utf8(key).expect("key is valid utf8 string"), std::str::from_utf8(value).expect("value is valid utf8 string"))))
}

fn parse_ping(input: &[u8]) -> IResult<&[u8], Command> {
    let (input, _) = tag_no_case("ping")(input)?;
    Ok((input, Command::Ping))
}

fn parse_quit(input: &[u8]) -> IResult<&[u8], Command> {
    let (input, _) = tag_no_case("exit")(input)?;
    Ok((input, Command::Quit))
}

pub fn parse_command(input: &[u8]) -> IResult<&[u8], Command> {
    alt((parse_db_size, parse_get, parse_set, parse_ping, parse_quit))(input)
}

pub fn try_parse_command(input: &[u8]) -> Option<Command> {
    match parse_command(input) {
        Ok((_, command)) => Some(command),
        Err(_) => None,
    }
}
