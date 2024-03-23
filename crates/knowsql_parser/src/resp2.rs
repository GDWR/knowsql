///! Redis Serialization Protocol (RESP2) parser
///! https://redis.io/docs/reference/protocol-spec/
use nom::{
    branch::alt,
    bytes::complete::{tag_no_case, take},
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

fn parse_string(input: &str) -> IResult<&str, Data> {
    let (input, _) = tag_no_case("+")(input)?;
    let (input, data) = not_line_ending(input)?;
    let (input, _) = line_ending(input)?;
    Ok((input, Data::String(data)))
}

fn parse_error(input: &str) -> IResult<&str, Data> {
    let (input, _) = tag_no_case("-")(input)?;
    let (input, data) = not_line_ending(input)?;
    let (input, _) = line_ending(input)?;
    Ok((input, Data::Error(data)))
}

fn parse_integer(input: &str) -> IResult<&str, Data> {
    let (input, _) = tag_no_case(":")(input)?;
    let (input, data) = digit1(input)?;
    let data = data
        .parse()
        .expect("string parsed with digit1 is a valid integer");
    let (input, _) = line_ending(input)?;
    Ok((input, Data::Integer(data)))
}

fn parse_bulk_string(input: &str) -> IResult<&str, Data> {
    let (input, _) = tag_no_case("$")(input)?;
    let (input, length) = digit1(input)?;
    let length = length
        .parse()
        .expect("string parsed with digit1 is a valid integer");
    let (input, _) = line_ending(input)?;
    let (input, data) = take(length)(input)?;
    let (input, _) = line_ending(input)?;
    Ok((input, Data::BulkString { length, data }))
}

fn parse_array<'a>(input: &str) -> IResult<&str, Data> {
    let (input, _) = tag_no_case("*")(input)?;
    let (input, length) = digit1(input)?;
    let length = length
        .parse()
        .expect("string parsed with digit1 is a valid integer");
    let (input, _) = line_ending(input)?;

    let (input, data) = count(parse_data, length)(input)?;
    Ok((input, Data::Array(data)))
}

pub fn parse_data(input: &str) -> IResult<&str, Data> {
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
        assert_eq!(parse_data("+OK\r\n"), Ok(("", Data::String("OK"))));
        assert_eq!(parse_data("-ERR\r\n"), Ok(("", Data::Error("ERR"))));
        assert_eq!(parse_data(":1000\r\n"), Ok(("", Data::Integer(1000))));
        assert_eq!(
            parse_data("$6\r\nfoobar\r\n"),
            Ok((
                "",
                Data::BulkString {
                    length: 6,
                    data: "foobar"
                }
            ))
        );
        assert_eq!(
            parse_data("*3\r\n+Foo\r\n-Bar\r\n:1000\r\n"),
            Ok((
                "",
                Data::Array(vec![
                    Data::String("Foo"),
                    Data::Error("Bar"),
                    Data::Integer(1000)
                ])
            ))
        );
    }
}
