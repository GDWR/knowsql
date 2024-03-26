mod config;

use knowsql_bitcask::BitCask;
use knowsql_parser::{
    command::{Command, SubCommand},
    parse_command,
    protocol::resp2::Data,
};
use regex::Regex;

use std::{
    io::{BufWriter, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
};
use tracing::{debug, error, info, span, trace, Level};

fn main() {
    tracing_subscriber::fmt::init();
    let config = config::get_config();

    let bitcask = BitCask::open(config.data_dir.clone().into()).expect("failed to open bitcask");
    let bitcask = Arc::new(Mutex::new(bitcask));

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

        let bitcask = bitcask.clone();
        std::thread::spawn(move || handle_client(stream, bitcask));
    }
}

fn handle_client(mut stream: TcpStream, bitcask: Arc<Mutex<BitCask>>) {
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
                                    Data::BulkString(name),
                                    Data::Array(doc.iter().map(|d| Data::BulkString(d)).collect()),
                                ]
                            })
                            .collect(),
                    );

                    let resp = response.as_str().expect("constructed from static values");
                    writer.write_all(resp.as_bytes()).unwrap();
                }
                Command::Echo(message) => {
                    writer
                        .write_all(format!("+{}\r\n", message).as_bytes())
                        .unwrap();
                }
                Command::Get(key) => {
                    let bitcask = bitcask.lock().unwrap();
                    match bitcask.get(key) {
                        Some(value) => writer
                            .write_all(format!("+{}\r\n", value).as_bytes())
                            .unwrap(),
                        None => writer.write_all(b"$-1\r\n").unwrap(),
                    }
                }
                Command::Keys(None) => {
                    let keys = bitcask.lock().unwrap().keys();
                    let response = Data::Array(
                        keys.iter()
                            .map(|key| Data::BulkString(key))
                            .collect(),
                    );

                    match response.as_str() {
                        Some(resp) => writer.write_all(resp.as_bytes()).unwrap(),
                        None => {
                            writer.write_all(b"-failed to list keys\r\n").unwrap();
                        }
                    }
                }
                Command::Keys(Some(pattern)) => {
                    let re = match Regex::new(pattern) {
                        Ok(re) => re,
                        Err(_) => {
                            trace!(pattern = pattern, "invalid regex pattern");
                            writer.write_all(b"-invalid regex pattern\r\n").unwrap();
                            writer.flush().unwrap();
                            continue;
                        }
                    };

                    let keys = bitcask.lock().unwrap().keys();
                    let response = Data::Array(
                        keys.iter()
                            .filter(|key| re.is_match(key))
                            .map(|key| Data::BulkString(key))
                            .collect(),
                    );

                    let resp = response.as_str().expect("constructed from static values");
                    writer.write_all(resp.as_bytes()).unwrap();
                }
                Command::Set(key, value) => match bitcask.lock().unwrap().put(key, value) {
                    Ok(_) => {
                        writer.write_all(b"+OK\r\n").unwrap();
                    }
                    Err(_) => {
                        writer
                            .write_all("-failed to set key value pair\r\n".as_bytes())
                            .unwrap();
                    }
                },
                Command::DbSize => {
                    let size = bitcask.lock().unwrap().keys().len();

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
