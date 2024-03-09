mod config;

use std::{
    io::{BufRead, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
    sync::{Arc, Mutex},
};

use knowsql_bitcask::BitCask;

use knowsql_parser::{parse_command, Command};

fn main() {
    let config = config::get_config();

    let bitcask = {
        let cask = BitCask::open(PathBuf::from(&config.data_dir)).unwrap();
        let mutex = Mutex::new(cask);
        Arc::new(mutex)
    };

    println!("Starting server on port {}", config.port);

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
                Command::Get(key) => match bitcask.lock().unwrap().get(key) {
                    Some(value) => stream.write_all((value + "\n").as_bytes()).unwrap(),
                    None => stream.write_all(b"NIL\n").unwrap(),
                },
                Command::Set(key, value) => match bitcask.lock().unwrap().put(key, value) {
                    Ok(_) => stream.write_all(b"OK\n").unwrap(),
                    Err(_) => stream.write_all(b"ERR\n").unwrap(),
                },
                Command::Exit => {
                    stream.write_all(b"BYE\n").unwrap();
                    break;
                }
            }
        } else {
            stream.write_all(b"INV\n").unwrap();
        }
    }
}
