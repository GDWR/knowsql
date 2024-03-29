use std::collections::HashMap;
use std::fs::File;
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
    fn get_active_file(&self) -> PathBuf {
        self.data_dir.join(format!("{}.data", self.active_file_id))
    }
    fn build_key_dir(&mut self) {
        let mut file = File::open(self.get_active_file()).unwrap();

        loop {
            const HEADER_SIZE: usize =
                size_of::<u32>() + size_of::<i64>() + size_of::<u32>() + size_of::<u32>();
            let mut buf = [0; HEADER_SIZE];

            match file.read_exact(&mut buf) {
                Ok(_) => (),
                Err(_) => break,
            }

            let _crc = u32::from_be_bytes(buf[..4].try_into().unwrap());
            let timestamp = i64::from_be_bytes(buf[4..12].try_into().unwrap());
            let key_size = u32::from_be_bytes(buf[12..16].try_into().unwrap());
            let value_size = u32::from_be_bytes(buf[16..20].try_into().unwrap());
            let key = &mut vec![0; key_size as usize];
            file.read_exact(key).unwrap();

            // Skip over value
            let pos = file.stream_position().expect("we just read from the file");
            file.seek(std::io::SeekFrom::Current(value_size as i64))
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

        let mut cask = BitCask {
            data_dir: data_dir,
            active_file_id: 0,
            key_dir: HashMap::new(),
        };

        // ensure active data file exists
        let active_file = cask.get_active_file();
        if !active_file.exists() {
            File::create(&active_file)?;
        }

        cask.build_key_dir();

        Ok(cask)
    }
    /// Get a value from the store
    pub fn get(&self, key: &str) -> Option<String> {
        let meta = self.key_dir.get(key)?;

        let mut file = File::open(self.get_active_file()).unwrap();

        file.seek(std::io::SeekFrom::Start(meta.value_position))
            .unwrap();
        let mut buf = vec![0; meta.value_size as usize];
        file.read_exact(&mut buf).unwrap();
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

        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(self.get_active_file())?;

        file.write_all(&e)?;

        let p = file.stream_position().unwrap();
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
