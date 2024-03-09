mod config;

use std::{
    io::{BufRead, Write},
    net::TcpListener,
    path::PathBuf,
};

use knowsql_bitcask::BitCask;

use knowsql_parser::{parse_command, Command};

fn main() {
    let config = config::get_config();

    println!("Starting server on port {}", config.port);

    let listener = TcpListener::bind(format!("0.0.0.0:{}", config.port)).unwrap();
    let mut bitcask = BitCask::open(PathBuf::from(config.data_dir)).unwrap();

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();

        let mut bufreader = std::io::BufReader::new(&stream);
        let mut buf = String::new();
        bufreader.read_line(&mut buf).unwrap();

        if let Some(command) = parse_command(&buf) {
            match command {
                Command::Get(key) => {
                    if let Some(value) = bitcask.get(key) {
                        stream.write_all(value.as_bytes()).unwrap();
                    }
                }
                Command::Set(key, value) => match bitcask.put(key, value) {
                    Ok(_) => stream.write_all(b"OK").unwrap(),
                    Err(_) => stream.write_all(b"Error").unwrap(),
                },
            }
        } else {
            stream.write_all(b"Invalid command").unwrap();
        }
    }
}
