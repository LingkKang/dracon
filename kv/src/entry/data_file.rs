use crate::fio;

use std::sync::Arc;
use std::sync::RwLock;

/// A file that stores the key-value data.
pub struct DataFile {
    /// Unique identifier of the file.
    id: Arc<RwLock<u64>>,

    /// Writing offset of the file.
    /// This is the position where the next entry will be written.
    offset: Arc<RwLock<u64>>,

    /// The IO manager which handles the I/O operations.
    io_manager: Box<dyn fio::IoManager>,
}

impl DataFile {
    /// Create a new data file.
    pub fn new(path: &std::path::Path, id: u64) -> crate::err::Result<Self> {
        let io_manager = crate::fio::file_io::FileIo::new(path);
        let io_manager = match io_manager {
            Ok(io_manager) => io_manager,
            Err(err) => return Err(err),
        };

        Ok(DataFile {
            id: Arc::new(RwLock::new(id)),
            offset: Arc::new(RwLock::new(0)),
            io_manager: Box::new(io_manager),
        })
    }

    /// Get the unique identifier [`DataFile::id`] of the file.
    pub fn id(&self) -> u64 {
        *self.id.read().unwrap()
    }

    /// Get the writing offset [`DataFile::offset`] of the file.
    pub fn offset(&self) -> u64 {
        *self.offset.read().unwrap()
    }

    /// Write data to the file.
    /// The writing offset [`DataFile::offset`] will also be updated.
    pub fn write(&self, data: &[u8]) -> crate::err::Result<u64> {
        let result = self.io_manager.write(data);
        match result {
            Ok(size) => {
                let mut offset = self.offset.write().unwrap();
                *offset += size;
                Ok(size)
            }
            Err(err) => Err(err),
        }
    }

    pub fn sync(&self) -> crate::err::Result<()> {
        self.io_manager.sync()
    }
}

/// Configuration for the key-value storage data file.
pub struct DataFileConfig {
    /// The directory where the data files are stored.
    pub data_path_dir: &'static std::path::Path,

    /// The maximum size of a data file.
    pub data_file_size: u64,

    /// Should the data be synced to disk immediately
    /// after each writing.
    pub write_sync_strategy: bool,
}
