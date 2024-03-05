use std::{
    io::{BufRead, Write},
    net::TcpListener,
    path::PathBuf,
};

use hoard::bitcask::BitCask;
use hoard::command::Command;

fn main() {
    let port = std::env::var("HOARD_PORT").unwrap_or("6379".to_string());

    let listener = TcpListener::bind( format!("0.0.0.0:{}", port)).unwrap();
    let mut bitcask = BitCask::open(PathBuf::from("./data")).unwrap();

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();

        let mut bufreader = std::io::BufReader::new(&stream);
        let mut buf = String::new();
        bufreader.read_line(&mut buf).unwrap();

        let s: Vec<Command> = buf
            .split_whitespace()
            .map(|x| Command::String(x))
            .collect::<Vec<Command>>();

        let command = Command::Array(&s);

        match command {
            Command::Array(
                [Command::String("set"), Command::String(key), Command::String(value)],
            ) => {
                bitcask.put(key, value).unwrap();
                stream.write_all(b"OK\r\n").unwrap();
            }
            Command::Array([Command::String("get"), Command::String(key)]) => {
                let value = bitcask.get(key).unwrap();
                stream.write_all(value.as_bytes()).unwrap();
                stream.write_all(b"\r\n").unwrap();
            }
            c => println!("Invalid command: {} {:?}", buf, c),
        }
    }
}
