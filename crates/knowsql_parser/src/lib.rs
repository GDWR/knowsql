use nom::{branch::alt, bytes::complete::{tag, tag_no_case}, IResult};

#[derive(Debug, PartialEq)]
pub enum Command<'a> {
    Get(&'a str),
    Set(&'a str, &'a str),
    Exit,
}

fn parse_set(input: &str) -> IResult<&str, Command> {
    let (input, _) = tag_no_case("set ")(input)?;
    let (input, key) = nom::character::complete::alpha1(input)?;
    let (input, _) = tag(" ")(input)?;
    let (input, value) = nom::character::complete::alpha1(input)?;
    Ok((input, Command::Set(key, value)))
}

fn parse_get(input: &str) -> IResult<&str, Command> {
    let (input, _) = tag_no_case("get ")(input)?;
    let (input, key) = nom::character::complete::alpha1(input)?;
    Ok((input, Command::Get(key)))
}

fn parse_exit(input: &str) -> IResult<&str, Command> {
    let (input, _) = tag_no_case("exit")(input)?;
    Ok((input, Command::Exit))
}

pub fn parse_command(input: &str) -> Option<Command> {
    match alt((parse_get, parse_set, parse_exit))(input) {
        Ok((_, command)) => Some(command),
        _ => None,
    }
}
