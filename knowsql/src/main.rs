mod config;

use knowsql_parser::{command::Command, parse_command};
use std::{
    collections::HashMap,
    io::{BufRead, BufReader, BufWriter, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
};
use tracing::{debug, error, info, span, Level};

fn main() {
    tracing_subscriber::fmt::init();
    let config = config::get_config();

    let map: HashMap<String, String> = HashMap::new();
    let map = Arc::new(Mutex::new(map));

    info!(
        port = config.port,
        data_dir = config.data_dir,
        "starting knowsql server"
    );

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = match TcpListener::bind(addr.clone()) {
        Ok(listener) => listener,
        Err(err) => {
            error!(addr = addr, err = %err, "failed to create TcpListener");
            return;
        }
    };

    for stream in listener.incoming() {
        let stream = match stream {
            Ok(stream) => stream,
            Err(err) => {
                error!(err = %err, "failed to accept client, continuing to serve next client");
                continue;
            }
        };

        let map = map.clone();
        std::thread::spawn(move || handle_client(stream, map));
    }
}

fn handle_client(stream: TcpStream, map: Arc<Mutex<HashMap<String, String>>>) {
    let _guard = span!(
        Level::INFO,
        "client",
        client_addr = stream
            .peer_addr()
            .expect("every client must have a peer_addr")
            .to_string(),
        thread = ?std::thread::current().id(),
    )
    .entered();

    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut writer = BufWriter::new(stream);

    info!("serving new client");
    loop {
        let buf_size = match reader.fill_buf() {
            Ok(buf) => buf.len(),
            Err(_) => break,
        };

        let str_buf = std::str::from_utf8(reader.buffer()).expect("client input is utf8");

        let (remaining, command) = match parse_command(str_buf) {
            Ok(c) => c,
            Err(err) => {
                error!(err = %err, "parsing did not complete, allowing buffer to refill");
                break 
            },
        };

        debug!(command = ?command, "handling command");

        match command {
            Command::Get(key) => {
                let map = map.lock().unwrap();
                match map.get(key) {
                    Some(value) => writer
                        .write_all(format!("+{}\r\n", value).as_bytes())
                        .unwrap(),
                    None => writer.write_all(b"$-1\r\n").unwrap(),
                }
            }
            Command::Set(key, value) => {
                let mut map = map.lock().unwrap();
                map.insert(key.to_string(), value.to_string());
                writer.write_all(b"+OK\r\n").unwrap();
            }
            Command::DbSize => {
                let map = map.lock().unwrap();
                let size = map.len();
                writer
                    .write_all(format!(":{}\r\n", size).as_bytes())
                    .unwrap();
            }
            Command::Quit => {
                debug!("client quitting");
                writer.write_all(b"+OK\r\n").unwrap();
                break;
            }
        }
        writer.flush().unwrap();

        let read = buf_size - remaining.len();
        reader.consume(read);
    }

    info!("client session closing");
}
