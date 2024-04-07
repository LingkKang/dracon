//! APIs that expose to the user to interact with the database.

use crate::entry::data_file::DataFile;
use crate::err::Err;
use crate::err::ErrCode;

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
    pub fn put(&self, key: &[u8], value: &[u8]) -> crate::err::Result<()> {
        if key.is_empty() {
            return Err(crate::err::Err {
                code: crate::err::ErrCode::EmptyKeyError,
                msg: "Key should not be empty".to_owned(),
            });
        }

        let mut entry = crate::entry::Entry {
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
        entry: &mut crate::entry::Entry,
    ) -> crate::err::Result<crate::entry::EntryPos> {
        let path = self.config.data_path_dir;

        // Encode the input data.
        let encoded_entry = entry.encode();
        let len = encoded_entry.len() as u64;

        // Get the current active file.
        let active_file = self.active_data_file.write();
        let mut active_file = match active_file {
            Ok(f) => f,
            Err(e) => {
                return Err(Err {
                    code: ErrCode::WriteDataFileFailed,
                    msg: "Failed to acquire write lock".to_owned()
                        + &e.to_string(),
                });
            }
        };

        // Check if the active file can hold the entry.
        if active_file.offset() + len > self.config.data_file_size {
            // If can not hold, then:
            // 1. Sync (save) the active file to disk.
            active_file.sync()?;

            // 2. Save the active file into the hash set of inactive files.
            let id = active_file.id();
            let older_data_files = self.older_data_files.write();
            let mut older_data_files = match older_data_files {
                Ok(f) => f,
                Err(e) => {
                    return Err(Err {
                        code: ErrCode::WriteDataFileFailed,
                        msg: "Failed to acquire write lock".to_owned()
                            + &e.to_string(),
                    });
                }
            };
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
}
