///! Entries represent the data that will be stored directly in the data file

#[derive(Clone, Debug)]
pub struct Entry<'a> {
    pub timestamp: i64,
    pub key_size: u32,
    pub value_size: u32,
    pub key: &'a str,
    pub value: &'a [u8],
}

impl Entry<'_> {
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.timestamp.to_be_bytes());
        buf.extend_from_slice(&self.key_size.to_be_bytes());
        buf.extend_from_slice(&self.value_size.to_be_bytes());
        buf.extend_from_slice(self.key.as_bytes());
        buf.extend_from_slice(self.value);
        buf
    }
}
