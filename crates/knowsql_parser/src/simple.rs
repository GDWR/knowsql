use crate::command::Command;

use nom::{
    branch::alt,
    bytes::streaming::{tag, tag_no_case},
    character::streaming::alphanumeric1,
    IResult,
};

fn parse_db_size(input: &str) -> IResult<&str, Command> {
    let (input, _) = tag_no_case("dbsize")(input)?;
    Ok((input, Command::DbSize))
}

fn parse_get(input: &str) -> IResult<&str, Command> {
    let (input, _) = tag_no_case("get")(input)?;
    let (input, _) = tag(" ")(input)?;
    let (input, key) = alphanumeric1(input)?;
    Ok((input, Command::Get(key)))
}

fn parse_set(input: &str) -> IResult<&str, Command> {
    let (input, _) = tag_no_case("set")(input)?;
    let (input, _) = tag(" ")(input)?;
    let (input, key) = alphanumeric1(input)?;
    let (input, _) = tag(" ")(input)?;
    let (input, value) = alphanumeric1(input)?;
    Ok((input, Command::Set(key, value)))
}

fn parse_quit(input: &str) -> IResult<&str, Command> {
    let (input, _) = tag_no_case("exit")(input)?;
    Ok((input, Command::Quit))
}

pub fn parse_command(input: &str) -> IResult<&str, Command> {
    alt((parse_db_size, parse_get, parse_set, parse_quit))(input)
}

pub fn try_parse_command(input: &str) -> Option<Command> {
    match parse_command(input) {
        Ok((_, command)) => Some(command),
        Err(_) => None,
    }
}
