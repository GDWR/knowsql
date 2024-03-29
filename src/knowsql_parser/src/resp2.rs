use crate::{
    command::{Command, SubCommand},
    protocol::resp2::{
        self,
        Data::{Array, BulkString},
    },
};

use nom::IResult;
use tracing::debug;

pub fn parse_command(input: &[u8]) -> IResult<&[u8], Command> {
    let (remaining, data) = resp2::parse_data(input)?;

    if let Array(arr) = data {
        match arr[..] {
            [BulkString("COMMAND"), BulkString("DOCS")] => {
                Ok((remaining, Command::Command(SubCommand::Docs)))
            }
            [BulkString("DBSIZE")] => Ok((remaining, Command::DbSize)),
            [BulkString("ECHO"), BulkString(data)] => Ok((remaining, Command::Echo(data))),
            [BulkString("GET"), BulkString(key)] => Ok((remaining, Command::Get(key))),
            [BulkString("SET"), BulkString(key), BulkString(value)] => {
                Ok((remaining, Command::Set(key, value.as_bytes())))
            }
            [BulkString("KEYS")] => Ok((remaining, Command::Keys(None))),
            [BulkString("KEYS"), BulkString(pattern)] => {
                Ok((remaining, Command::Keys(Some(pattern))))
            }
            [BulkString("PING")] => Ok((remaining, Command::Ping)),
            [BulkString("QUIT")] => Ok((remaining, Command::Quit)),
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
