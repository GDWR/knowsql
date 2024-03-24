use crate::{
    command::{Command, SubCommand},
    protocol::resp2,
};

use nom::IResult;
use tracing::debug;

pub fn parse_command(input: &[u8]) -> IResult<&[u8], Command> {
    let (remaining, data) = resp2::parse_data(input)?;

    if let resp2::Data::Array(arr) = data {
        match arr[..] {
            [resp2::Data::BulkString {
                data: "COMMAND", ..
            }, resp2::Data::BulkString { data: "DOCS", .. }] => {
                Ok((remaining, Command::Command(SubCommand::Docs)))
            }
            [resp2::Data::BulkString { data: "DBSIZE", .. }] => Ok((remaining, Command::DbSize)),
            [resp2::Data::BulkString { data: "GET", .. }, resp2::Data::BulkString { data: key, .. }] => {
                Ok((remaining, Command::Get(key)))
            }
            [resp2::Data::BulkString { data: "SET", .. }, resp2::Data::BulkString { data: key, .. }, resp2::Data::BulkString { data: value, .. }] => {
                Ok((remaining, Command::Set(key, value)))
            }
            [resp2::Data::BulkString { data: "KEYS", .. }, resp2::Data::BulkString { data: pattern, .. }] => {
                Ok((remaining, Command::Keys(pattern)))
            }
            [resp2::Data::BulkString { data: "PING", .. }] => Ok((remaining, Command::Ping)),
            [resp2::Data::BulkString { data: "QUIT", .. }] => Ok((remaining, Command::Quit)),
            _ => {
                debug!("Failed to parse command: {:?}", arr);
                Err(nom::Err::Error(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::Tag,
                )))
            }
        }
    } else {
        Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )))
    }
}

pub fn try_parse_command(input: &[u8]) -> Option<Command> {
    match parse_command(input) {
        Ok((_, command)) => Some(command),
        Err(_) => None,
    }
}
