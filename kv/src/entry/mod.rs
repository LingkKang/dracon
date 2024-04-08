//! In the bitcask storage model, the key-value store
//! is a collection of files, each of which is a log
//! of key-value pairs. The key-value pairs are stored
//! in the order they are written to the file.

pub mod data_file;

/// Position of an entry.
#[derive(Clone, Copy)]
pub struct EntryPos {
    pub(crate) file_id: u64,
    pub(crate) offset: u64,
}

impl std::fmt::Debug for EntryPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EntryPos(file_id={}, offset={})", self.file_id, self.offset)
    }
}

/// The type of an entry.
#[allow(dead_code)]
#[derive(Clone)]
pub enum EntryType {
    /// Normal entry.
    Normal = 1,

    /// Deleted entry.
    Deleted = 2,
}

/// Data entry.
/// Represents the actual data that is written to data file.
pub struct Entry {
    pub(crate) key: Vec<u8>,
    pub(crate) val: Vec<u8>,
    pub(crate) typ: EntryType,
}

impl Entry {
    /// Encode the entry into a byte array.
    pub fn encode(&mut self) -> Vec<u8> {
        let mut encoded = Vec::new();
        encoded.push(self.key.len() as u8);
        encoded.push(self.val.len() as u8);
        encoded.extend_from_slice(&self.key);
        encoded.extend_from_slice(&self.val);
        encoded.push(self.typ.clone() as u8);
        encoded
    }

    pub fn decode(encoded: &[u8]) -> Self {
        let key_len = encoded[0] as usize;
        let val_len = encoded[1] as usize;
        let key = encoded[2..2 + key_len].to_vec();
        let val = encoded[2 + key_len..2 + key_len + val_len].to_vec();
        let typ = match encoded[2 + key_len + val_len] {
            1 => EntryType::Normal,
            2 => EntryType::Deleted,
            _ => panic!("Invalid entry type"),
        };
        Entry { key, val, typ }
    }
}
