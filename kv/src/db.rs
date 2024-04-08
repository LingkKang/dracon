//! APIs that expose to the user to interact with the database.

use crate::entry::data_file::DataFile;
use crate::entry::Entry;
use crate::err::Err;
use crate::err::ErrCode;
use crate::err::Result;

use std::sync::Arc;

/// The storage engine.
pub struct Engine {
    config: Arc<crate::entry::data_file::DataFileConfig>,
    active_data_file: Arc<std::sync::RwLock<DataFile>>,
    older_data_files:
        Arc<std::sync::RwLock<std::collections::HashMap<u64, DataFile>>>,
    indices: Box<dyn crate::index::Indexer>,
}

impl Engine {
    /// Save a key-value data pair.
    /// `key` should not be empty.
    pub fn put(&self, key: &[u8], value: &[u8]) -> Result<()> {
        if key.is_empty() {
            return Err(Err {
                code: ErrCode::EmptyKeyError,
                msg: "Key should not be empty".to_owned(),
            });
        }

        let mut entry = Entry {
            key: key.to_vec(),
            val: value.to_vec(),
            typ: crate::entry::EntryType::Normal,
        };

        // Append the entry to the active data file.
        let key_position = self.append_entry(&mut entry)?;

        // Put the key into the in-memory index.
        if self.indices.put(key.to_vec(), key_position) {
            Ok(())
        } else {
            Err(Err {
                code: ErrCode::IndexUpdateFailed,
                msg: format!(
                    "Failed to update in-memory index of key {:?} at {:?}",
                    key, key_position
                ),
            })
        }
    }

    /// Append an entry to the current active data file.
    fn append_entry(
        &self,
        entry: &mut Entry,
    ) -> Result<crate::entry::EntryPos> {
        let path = self.config.data_path_dir;

        // Encode the input data.
        let encoded_entry = entry.encode();
        let len = encoded_entry.len() as u64;

        // Get the current active file.
        let active_file = self.active_data_file.write();
        let mut active_file = crate::err::match_write_lock!(active_file);

        // Check if the active file can hold the entry.
        if active_file.offset() + len > self.config.data_file_size {
            // If can not hold, then:
            // 1. Sync (save) the active file to disk.
            active_file.sync()?;

            // 2. Save the active file into the hash set of inactive files.
            let id = active_file.id();
            let older_data_files = self.older_data_files.write();
            let mut older_data_files =
                crate::err::match_write_lock!(older_data_files);

            let data_file = DataFile::new(path, id)?;
            older_data_files.insert(id, data_file);

            // 3. Create a new active file and replace it with the previous one.
            let new_id = id + 1;
            let new_active_file = DataFile::new(path, new_id)?;
            *active_file = new_active_file;
        }

        // Write (append) the entry to the active file.
        // In case that the active file is changed, get writing offset again.
        let write_offset = active_file.offset();
        active_file.write(&encoded_entry)?;

        // Sync the active file if in need.
        if self.config.write_sync_strategy {
            active_file.sync()?;
        }

        Ok(crate::entry::EntryPos {
            file_id: active_file.id(),
            offset: write_offset,
        })
    }

    /// Get the value of a key.
    pub fn get(&self, key: &[u8]) -> Result<Entry> {
        // Key should not be empty.
        if key.is_empty() {
            return Err(Err {
                code: ErrCode::EmptyKeyError,
                msg: "Key should not be empty".to_owned(),
            });
        }

        // Get the position of the key `EntryPos` from the in-memory index.
        let pos = match self.indices.get(key.to_vec()) {
            Some(p) => p,
            None => {
                return Err(Err {
                    code: ErrCode::KeyNotFoundError,
                    msg: format!("Key {:?} not found", key),
                });
            }
        };

        let id = pos.file_id;
        let offset = pos.offset;

        // Acquire read lock on the active file and the older files.
        let active = crate::err::match_read_lock!(self.active_data_file.read());
        let old = crate::err::match_read_lock!(self.older_data_files.read());

        // Check where the data is stored.
        match active.id() == id {
            true => {
                // The data is store in the active file.
                // Read the data from the active file.
                let data = active.read(offset)?;
                // Decode the data into an entry and return it.
                Ok(Entry::decode(&data))
            }
            false => {
                // The data is stored in one of the inactive files.
                let old_file = match old.get(&id) {
                    Some(f) => f,
                    None => {
                        return Err(Err {
                            code: ErrCode::ReadDataFileFailed,
                            msg: format!("File with id {} not found", id),
                        });
                    }
                };
                let data = old_file.read(offset)?;
                Ok(Entry::decode(&data))
            }
        }
    }
}
