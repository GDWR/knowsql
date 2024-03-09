use std::{
    fs,
    io::{BufRead, Write},
    net::TcpListener,
    path::PathBuf,
};

use knowsql_bitcask::BitCask;
use knowsql::command::Command;
use serde::Deserialize;

const DEFAULT_CONFIG_PATH: &'static str = "/etc/knowsql/config.toml";

#[derive(Debug, Deserialize)]
pub struct Config {
    data_dir: String,
    port: usize,
}
impl Default for Config {
    fn default() -> Self {
        Config {
            data_dir: "./data".to_string(),
            port: 2288,
        }
    }
}

/// Use environment variable KNOWSQL_CONFIG or DEFAULT_CONFIG_PATH to load the config
///   if it doesn't exist, use default config
fn get_config() -> Config {
    let config_path = std::env::var("KNOWSQL_CONFIG").unwrap_or(DEFAULT_CONFIG_PATH.to_string());

    if let Ok(config) = fs::read_to_string(config_path) {
        return toml::from_str(&config).unwrap();
    } else {
        println!("Missing configuration, using defaults");
        return Config::default();
    }
}

fn main() {
    let config = get_config();

    println!("Starting server on port {}", config.port);

    let listener = TcpListener::bind(format!("0.0.0.0:{}", config.port)).unwrap();
    let mut bitcask = BitCask::open(PathBuf::from(config.data_dir)).unwrap();

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
