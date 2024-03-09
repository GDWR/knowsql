use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, Write};
use std::path::PathBuf;

use chrono::Utc;

#[derive(Debug)]
pub struct BitCask {
    data_dir: PathBuf,
    active_file_id: u32,
    key_dir: HashMap<String, Key>,
}

/// A entry that will exist within a data file
#[derive(Clone, Debug)]
struct Entry<'a> {
    crc: u32,
    timestamp: i64,
    key_size: u32,
    value_size: u32,
    key: &'a str,
    value: &'a str,
}

/// A key to locate a value within a data file
#[derive(Debug)]
struct Key {
    file_id: u32,
    value_size: u32,
    value_position: u64,
    timestamp: i64,
}

impl Entry<'_> {
    fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.crc.to_be_bytes());
        buf.extend_from_slice(&self.timestamp.to_be_bytes());
        buf.extend_from_slice(&self.key_size.to_be_bytes());
        buf.extend_from_slice(&self.value_size.to_be_bytes());
        buf.extend_from_slice(self.key.as_bytes());
        buf.extend_from_slice(self.value.as_bytes());
        buf
    }
}

impl BitCask {
    fn get_active_file(&self) -> PathBuf {
        self.data_dir.join(format!("{}.data", self.active_file_id))
    }

    /// Open a BitCask store
    ///   if provided data_dir does not exist it will be created or an error will be returned
    pub fn open(data_dir: PathBuf) -> std::io::Result<BitCask> {
        if !data_dir.exists() {
            std::fs::create_dir(&data_dir)?;
        }

        Ok(BitCask {
            data_dir: data_dir,
            active_file_id: 0,
            key_dir: HashMap::new(),
        })
    }
    /// Get a value from the store
    pub fn get(&mut self, key: &str) -> Option<String> {
        let meta = self.key_dir.get(key)?;

        let mut file = File::open(self.get_active_file()).unwrap();

        file.seek(std::io::SeekFrom::Start(meta.value_position))
            .unwrap();
        let mut buf = vec![0; meta.value_size as usize];
        file.read_exact(&mut buf).unwrap();
        Some(String::from_utf8(buf).unwrap())
    }
    /// Put a key-value pair into the store
    pub fn put(&mut self, key: &str, value: &str) -> std::io::Result<()> {
        let entry = Entry {
            crc: 0,
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
