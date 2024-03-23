mod config;

use std::{
    io::{BufRead, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
    sync::{Arc, Mutex},
};

use knowsql_bitcask::BitCask;

use knowsql_parser::{parse_command, Command, KeyValue};

use tracing::info;

fn main() {
    tracing_subscriber::fmt::init();
    let config = config::get_config();

    let bitcask = {
        let cask = BitCask::open(PathBuf::from(&config.data_dir)).unwrap();
        let mutex = Mutex::new(cask);
        Arc::new(mutex)
    };

    info!(
        port = config.port,
        data_dir = config.data_dir,
        "starting knowsql server"
    );

    let listener = TcpListener::bind(format!("0.0.0.0:{}", config.port)).unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let bitcask = bitcask.clone();
        std::thread::spawn(move || handle_client(stream, bitcask));
    }
}

fn handle_client(mut stream: TcpStream, bitcask: Arc<Mutex<BitCask>>) {
    let mut bufreader = std::io::BufReader::new(stream.try_clone().unwrap());
    loop {
        let mut buf = String::new();
        bufreader.read_line(&mut buf).unwrap();

        if let Some(command) = parse_command(&buf) {
            match command {
                Command::Set(KeyValue { key, value }) => {
                    match bitcask.lock().unwrap().put(key, value) {
                        Ok(_) => stream.write_all(b"OK\n").unwrap(),
                        Err(_) => stream.write_all(b"ERR\n").unwrap(),
                    }
                }
                Command::MSet(key_values) => {
                    let mut cask = bitcask.lock().unwrap();
                    for kv in key_values {
                        cask.put(kv.key, kv.value).unwrap();
                    }
                    stream.write_all(b"OK\n").unwrap();
                }
                Command::Get(key) => match bitcask.lock().unwrap().get(key) {
                    Some(value) => stream.write_all((value + "\n").as_bytes()).unwrap(),
                    None => stream.write_all(b"NIL\n").unwrap(),
                },
                Command::MGet(keys) => {
                    let mut cask = bitcask.lock().unwrap();

                    for (i, key) in keys.iter().enumerate() {
                        if i > 0 {
                            stream.write(" ".as_bytes()).unwrap();
                        }

                        if let Some(value) = cask.get(&key) {
                            stream.write(value.as_bytes()).unwrap();
                        } else {
                            stream.write("NIL".as_bytes()).unwrap();
                        }
                    }
                    stream.write("\n".as_bytes()).unwrap();
                }
                Command::List => {
                    let keys = bitcask.lock().unwrap().list_keys().join(" ");
                    stream.write_all((keys + "\n").as_bytes()).unwrap();
                }
                Command::Exit => {
                    stream.write_all(b"BYE\n").unwrap();
                    break;
                }
                Command::Increment(key) => {
                    let mut cask = bitcask.lock().unwrap();
                    let value = cask.get(&key).unwrap_or("0".to_string());

                    if let Ok(current_value) = value.parse::<isize>() {
                        let new_value = (current_value + 1).to_string();
                        cask.put(&key, &new_value).unwrap();
                        stream.write_all((new_value + "\n").as_bytes()).unwrap();
                    } else {
                        stream.write_all(b"ERR\n").unwrap();
                    }
                }
            }
        } else {
            stream.write_all(b"INV\n").unwrap();
        }
    }
}
