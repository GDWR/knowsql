mod config;

use knowsql_parser::{
    command::{Command, SubCommand},
    parse_command,
    protocol::resp2::Data,
};

use std::{
    collections::HashMap,
    io::{BufWriter, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
};
use tracing::{debug, error, info, span, trace, Level};

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

fn handle_client(mut stream: TcpStream, map: Arc<Mutex<HashMap<String, String>>>) {
    let _guard = span!(
        Level::INFO,
        "client",
        addr = stream
            .peer_addr()
            .expect("every client must have a peer_addr")
            .to_string(),
    )
    .entered();

    let mut buffer = [0; 1024 * 1024];
    let mut writer = BufWriter::new(stream.try_clone().unwrap());

    info!("new connection");
    loop {
        trace!("reading from stream");
        let read = match stream.read(&mut buffer) {
            Err(err) => {
                debug!(err = %err, "failed to read from stream");
                break;
            }
            Ok(read) if read == 0 => {
                debug!("client closed connection");
                break;
            }
            Ok(read) => read,
        };

        let mut consumed = 0;
        while consumed <= read {
            let (remaining, command) = match parse_command(&buffer[consumed..read]) {
                Ok(c) => c,
                Err(_) => {
                    trace!("parsing did not complete, allowing buffer to refill");
                    break;
                }
            };

            let size = (read - consumed) - remaining.len();
            consumed += size;
            debug!(command = ?command, size = size, "handling command");

            match command {
                Command::Command(SubCommand::Docs) => {
                    let response = Data::Array(
                        Command::all_commands()
                            .iter()
                            .flat_map(|(name, doc)| {
                                vec![
                                    Data::BulkString {
                                        data: name,
                                        length: name.len(),
                                    },
                                    Data::Array(vec![Data::BulkString {
                                        data: doc[0],
                                        length: doc[0].len(),
                                    }]),
                                ]
                            })
                            .collect(),
                    );

                    let resp = response.as_str().expect("constructed from static values");

                    trace!(data = ?response, resp = resp, "sending response");
                    writer.write_all(resp.as_bytes()).unwrap();
                }
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
                Command::Ping => {
                    writer.write_all(b"+PONG\r\n").unwrap();
                }
                Command::Quit => {
                    debug!("client quitting");
                    writer.write_all(b"+OK\r\n").unwrap();
                    break;
                }
            }

            trace!("flushing writer");
            writer.flush().unwrap();
        }

        buffer.copy_within(consumed..read, 0);
    }

    debug!("client going away");
}
