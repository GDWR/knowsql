use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, Write};
use std::mem::size_of;
use std::path::PathBuf;

use chrono::Utc;

mod entry;
use entry::Entry;

#[derive(Debug)]
pub struct BitCask {
    data_dir: PathBuf,
    active_file_id: u32,
    key_dir: HashMap<String, Key>,

    write_handle: File,
    read_handle: File,
}

/// A key to locate a value within a data file
#[derive(Debug)]
struct Key {
    #[allow(dead_code)]
    file_id: u32,
    value_size: u32,
    value_position: u64,
    #[allow(dead_code)]
    timestamp: i64,
}

impl BitCask {
    fn build_key_dir(&mut self) {
        self.read_handle.rewind().unwrap();

        loop {
            const HEADER_SIZE: usize = size_of::<i64>() + size_of::<u32>() + size_of::<u32>();
            let mut buf = [0; HEADER_SIZE];

            match self.read_handle.read_exact(&mut buf) {
                Ok(_) => (),
                Err(_) => break,
            }

            let timestamp = i64::from_be_bytes(buf[..8].try_into().unwrap());
            let key_size = u32::from_be_bytes(buf[8..12].try_into().unwrap());
            let value_size = u32::from_be_bytes(buf[12..16].try_into().unwrap());
            let key = &mut vec![0; key_size as usize];
            self.read_handle.read_exact(key).unwrap();

            // Skip over value
            let pos = self
                .read_handle
                .stream_position()
                .expect("we just read from the file");
            self.read_handle
                .seek(std::io::SeekFrom::Current(value_size as i64))
                .expect("content is not malformed");

            self.key_dir.insert(
                String::from_utf8(key.to_vec()).unwrap(),
                Key {
                    file_id: self.active_file_id,
                    value_size: value_size,
                    value_position: pos,
                    timestamp: timestamp,
                },
            );
        }
    }

    /// Open a BitCask store
    ///   if provided data_dir does not exist it will be created or an error will be returned
    pub fn open(data_dir: PathBuf) -> std::io::Result<BitCask> {
        if !data_dir.exists() {
            std::fs::create_dir(&data_dir)?;
        }

        let active_file = data_dir.join("0.data");
        let write_handle = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&active_file)?;
        let read_handle = OpenOptions::new().read(true).open(&active_file)?;

        let mut cask = BitCask {
            data_dir,
            active_file_id: 0,
            key_dir: HashMap::new(),
            write_handle,
            read_handle,
        };

        cask.build_key_dir();

        Ok(cask)
    }
    /// Get a value from the store
    pub fn get(&mut self, key: &str) -> Option<String> {
        let meta = self.key_dir.get(key)?;

        self.read_handle
            .seek(std::io::SeekFrom::Start(meta.value_position))
            .unwrap();

        let mut buf = vec![0; meta.value_size as usize];
        self.read_handle.read_exact(&mut buf).unwrap();
        Some(String::from_utf8(buf).unwrap())
    }
    /// Put a key-value pair into the store
    pub fn put(&mut self, key: &str, value: &[u8]) -> std::io::Result<()> {
        let entry = Entry {
            timestamp: Utc::now().timestamp(),
            key_size: key.len() as u32,
            value_size: value.len() as u32,
            key: key,
            value: value,
        };

        let e = entry.serialize();

        self.write_handle.write(&e)?;
        self.write_handle.flush()?;
        let p = self.write_handle.stream_position().unwrap();

        self.key_dir.insert(
            key.to_string(),
            Key {
                file_id: 0,
                value_size: value.len() as u32,
                value_position: p - entry.value_size as u64,
                timestamp: entry.timestamp,
            },
        );

        Ok(())
    }
    /// Delete a key from the store
    pub fn delete(&mut self, key: &str) -> Option<()> {
        self.key_dir.remove(key).map(|_| ())
    }
    /// Alias for [`BitCask::list_keys()`]
    pub fn keys(&self) -> Vec<String> {
        self.list_keys()
    }
    /// List all keys in the store
    pub fn list_keys(&self) -> Vec<String> {
        self.key_dir.keys().cloned().collect()
    }
}
