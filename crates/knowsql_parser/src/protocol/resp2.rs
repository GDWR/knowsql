///! Redis Serialization Protocol (RESP2) parser
///! https://redis.io/docs/reference/protocol-spec/
use nom::{
    branch::alt,
    bytes::streaming::{tag_no_case, take},
    character::complete::{digit1, line_ending, not_line_ending},
    multi::count,
    IResult,
};

#[derive(Debug, PartialEq)]
pub enum Data<'a> {
    /// [Simple string](https://redis.io/docs/reference/protocol-spec/#simple-strings)
    String(&'a str),
    /// [Simple Error](https://redis.io/docs/reference/protocol-spec/#simple-errors)
    Error(&'a str),
    /// [Integer](https://redis.io/docs/reference/protocol-spec/#integers)
    Integer(i64),
    /// [Bulk String](https://redis.io/docs/reference/protocol-spec/#bulk-strings)
    BulkString { length: usize, data: &'a str },
    /// [Array](https://redis.io/docs/reference/protocol-spec/#arrays)
    Array(Vec<Data<'a>>),
}

impl Data<'_> {
    /// Convert the data to a RESP2 string
    pub fn as_str(&self) -> Option<String> {
        match self {
            Data::String(data) => Some(format!("+{}\r\n", data)),
            Data::Error(data) => Some(format!("-{}\r\n", data)),
            Data::Integer(data) => Some(format!(":{}\r\n", data)),
            Data::BulkString { data, .. } => Some(format!("${}\r\n{}\r\n", data.len(), data)),
            Data::Array(data) => {
                let mut result = String::from("*");
                result.push_str(&data.len().to_string());
                result.push_str("\r\n");
                for item in data {
                    if let Some(item_str) = item.as_str() {
                        result.push_str(&item_str);
                    } else {
                        return None;
                    }
                }
                Some(result)
            }
        }
    }
}

fn parse_string(input: &[u8]) -> IResult<&[u8], Data> {
    let (input, _) = tag_no_case("+")(input)?;
    let (input, data) = not_line_ending(input)?;
    let (input, _) = line_ending(input)?;
    Ok((
        input,
        Data::String(std::str::from_utf8(data).expect("data is valid utf8 string")),
    ))
}

fn parse_error(input: &[u8]) -> IResult<&[u8], Data> {
    let (input, _) = tag_no_case("-")(input)?;
    let (input, data) = not_line_ending(input)?;
    let (input, _) = line_ending(input)?;
    Ok((
        input,
        Data::Error(std::str::from_utf8(data).expect("data is valid utf8 string")),
    ))
}

fn parse_integer(input: &[u8]) -> IResult<&[u8], Data> {
    let (input, _) = tag_no_case(":")(input)?;
    let (input, data) = digit1(input)?;

    // safety: digit1 ensures that the string is valid utf8
    let data = unsafe {
        std::str::from_utf8_unchecked(data)
            .parse()
            .expect("string parsed with digit1 is a valid integer")
    };

    let (input, _) = line_ending(input)?;
    Ok((input, Data::Integer(data)))
}

fn parse_bulk_string(input: &[u8]) -> IResult<&[u8], Data> {
    let (input, _) = tag_no_case("$")(input)?;
    let (input, length) = digit1(input)?;

    // safety: digit1 ensures that the string is valid utf8
    let length = unsafe {
        std::str::from_utf8_unchecked(length)
            .parse()
            .expect("string parsed with digit1 is a valid integer")
    };

    let (input, _) = line_ending(input)?;
    let (input, data) = take(length)(input)?;
    let (input, _) = line_ending(input)?;
    Ok((
        input,
        Data::BulkString {
            length,
            data: std::str::from_utf8(data).expect("data is valid utf8 string"),
        },
    ))
}

fn parse_array<'a>(input: &[u8]) -> IResult<&[u8], Data> {
    let (input, _) = tag_no_case("*")(input)?;
    let (input, length) = digit1(input)?;

    // safety: digit1 ensures that the string is valid utf8
    let length = unsafe {
        std::str::from_utf8_unchecked(length)
            .parse()
            .expect("string parsed with digit1 is a valid integer")
    };

    let (input, _) = line_ending(input)?;

    let (input, data) = count(parse_data, length)(input)?;
    Ok((input, Data::Array(data)))
}

pub fn parse_data(input: &[u8]) -> IResult<&[u8], Data> {
    alt((
        parse_string,
        parse_error,
        parse_integer,
        parse_bulk_string,
        parse_array,
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_data() {
        assert_eq!(
            parse_data("+OK\r\n".as_bytes()),
            Ok(("".as_bytes(), Data::String("OK")))
        );
        assert_eq!(
            parse_data("-ERR\r\n".as_bytes()),
            Ok(("".as_bytes(), Data::Error("ERR")))
        );
        assert_eq!(
            parse_data(":1000\r\n".as_bytes()),
            Ok(("".as_bytes(), Data::Integer(1000)))
        );
        assert_eq!(
            parse_data("$6\r\nfoobar\r\n".as_bytes()),
            Ok((
                "".as_bytes(),
                Data::BulkString {
                    length: 6,
                    data: "foobar"
                }
            ))
        );
        assert_eq!(
            parse_data("*3\r\n+Foo\r\n-Bar\r\n:1000\r\n".as_bytes()),
            Ok((
                "".as_bytes(),
                Data::Array(vec![
                    Data::String("Foo"),
                    Data::Error("Bar"),
                    Data::Integer(1000)
                ])
            ))
        );
        assert_eq!(
            parse_data("*2\r\n$3\r\nGET\r\n$5\r\nhello\r\n\r\n*2\r\n$4\r\nECHO\r\n$20\r\n".as_bytes()),
            Ok((
                "\r\n*2\r\n$4\r\nECHO\r\n$20\r\n".as_bytes(),
                Data::Array(vec![
                    Data::BulkString{ data: "GET", length: 3 },
                    Data::BulkString{ data: "hello", length: 5 },
                ])
            ))
        )

    }
}
