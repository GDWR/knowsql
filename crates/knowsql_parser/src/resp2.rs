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
            [resp2::Data::BulkString("COMMAND"), resp2::Data::BulkString("DOCS")] => {
                Ok((remaining, Command::Command(SubCommand::Docs)))
            }
            [resp2::Data::BulkString("DBSIZE")] => Ok((remaining, Command::DbSize)),
            [resp2::Data::BulkString("ECHO"), resp2::Data::BulkString(data)] => {
                Ok((remaining, Command::Echo(data)))
            }
            [resp2::Data::BulkString("GET"), resp2::Data::BulkString(key)] => {
                Ok((remaining, Command::Get(key)))
            }
            [resp2::Data::BulkString("SET"), resp2::Data::BulkString(key), resp2::Data::BulkString(value)] => {
                Ok((remaining, Command::Set(key, value)))
            }
            [resp2::Data::BulkString("KEYS")] => Ok((remaining, Command::Keys(None))),
            [resp2::Data::BulkString("KEYS"), resp2::Data::BulkString(pattern)] => {
                Ok((remaining, Command::Keys(Some(pattern))))
            }
            [resp2::Data::BulkString("PING")] => Ok((remaining, Command::Ping)),
            [resp2::Data::BulkString("QUIT")] => Ok((remaining, Command::Quit)),
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
