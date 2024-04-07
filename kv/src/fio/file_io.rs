use super::IoManager;
use crate::err::Err;
use crate::err::ErrCode;
use crate::err::Result;

use std::io::Read;
use std::io::Seek;
use std::io::Write;

pub struct FileIo {
    file: std::sync::Arc<std::sync::RwLock<std::fs::File>>,
}

impl FileIo {
    pub fn new(file_name: &std::path::Path) -> Result<Self> {
        match std::fs::OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(file_name)
        {
            Ok(file) => Ok(FileIo {
                file: std::sync::Arc::new(std::sync::RwLock::new(file)),
            }),
            Err(err) => {
                log::error!("Failed to open data file: {}", err);
                Err(Err {
                    code: ErrCode::OpenDataFileFailed,
                    msg: err.to_string(),
                })
            }
        }
    }
}

impl IoManager for FileIo {
    fn read(&self, buf: &mut [u8], offset: u64) -> Result<u64> {
        // Acquire a read lock on the file.
        let read_guard = self.file.read();
        let read_guard = match read_guard {
            Ok(guard) => guard,
            Err(e) => {
                log::error!("Failed to acquire read lock on file: {}", e);
                return Err(Err {
                    code: ErrCode::ReadDataFileFailed,
                    msg: "Failed to acquire read lock".to_owned()
                        + &e.to_string(),
                });
            }
        };

        // Clone the file instance to read data.
        // TODO: Do I really need to clone the file descriptor?
        let mut file_inst = read_guard
            .try_clone()
            .map_err(|e| {
                log::error!("Failed to clone file descriptor: {}", e);
                Err {
                    code: ErrCode::ReadDataFileFailed,
                    msg: "Failed to clone file descriptor ".to_owned()
                        + &e.to_string(),
                }
            })
            .unwrap();

        // Seek to the offset.
        file_inst
            .seek(std::io::SeekFrom::Start(offset))
            .map_err(|e| {
                log::error!("Failed to seek in file: {}", e);
                Err {
                    code: ErrCode::ReadDataFileFailed,
                    msg: "Failed to seek in file ".to_owned() + &e.to_string(),
                }
            })
            .unwrap();

        // Read data from the file.
        let read_bytes = file_inst
            .read(buf)
            .map_err(|e| {
                log::error!("Read data file err: {}", e);
                Err { code: ErrCode::ReadDataFileFailed, msg: e.to_string() }
            })
            .unwrap();

        Ok(read_bytes as u64)
    }

    fn write(&self, buf: &[u8]) -> Result<u64> {
        // Acquire a write lock on the file.
        let write_guard = self.file.write();
        let mut write_guard = match write_guard {
            Ok(guard) => guard,
            Err(e) => {
                log::error!("Failed to acquire write lock on file: {}", e);
                return Err(Err {
                    code: ErrCode::WriteDataFileFailed,
                    msg: "Failed to acquire write lock ".to_owned()
                        + &e.to_string(),
                });
            }
        };

        // Write data to the file.
        match write_guard.write(buf) {
            Ok(size) => Ok(size as u64),
            Err(e) => {
                log::error!("Failed to write file: {}", e);
                Err(Err {
                    code: ErrCode::WriteDataFileFailed,
                    msg: e.to_string(),
                })
            }
        }
    }

    /// Sync the file to disk.
    /// Basically a wrapper around [`std::fs::File::sync_all()`].
    fn sync(&self) -> Result<()> {
        // Acquire a read lock on the file.
        let read_guard = self.file.read();
        let read_guard = match read_guard {
            Ok(guard) => guard,
            Err(e) => {
                log::error!("Failed to acquire read lock on file: {}", e);
                return Err(Err {
                    code: ErrCode::SyncDataFileFailed,
                    msg: "Failed to acquire read lock ".to_owned()
                        + &e.to_string(),
                });
            }
        };

        // Sync the file.
        if let Err(e) = read_guard.sync_all() {
            log::error!("Failed to sync file: {}", e);
            return Err(Err {
                code: ErrCode::SyncDataFileFailed,
                msg: e.to_string(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    /// Write data to the file and assert the write result.
    macro_rules! write_data_and_assert {
        ($data: expr, $fio: expr) => {
            let result = $fio.write($data.clone().as_bytes());
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), $data.len() as u64);
        };
    }

    #[test]
    fn test_file_io_write() {
        let file_path = std::path::Path::new("./test_write.data");

        // Create a file io instance
        let fio = FileIo::new(file_path);
        assert!(fio.is_ok());
        let fio = fio.unwrap();

        let key1 = "key1";
        write_data_and_assert!(key1, fio);

        let key2 = "key_abcd";
        write_data_and_assert!(key2, fio);

        // Clean up
        let del = std::fs::remove_file(file_path);
        assert!(del.is_ok());
    }

    /// Read data from the file and assert the read data.
    macro_rules! read_data_and_assert {
        ($len: expr, $offset: expr, $fio: expr) => {
            let mut buf = Vec::with_capacity($len);
            buf.resize($len, 0);
            let result = $fio.read(&mut buf, $offset);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), $len as u64);
            // Print the read data in decimal and in string.
            println!(
                "buf: {:?}\nval: {:?}\n",
                buf,
                std::str::from_utf8(&buf).unwrap()
            );
        };
    }

    #[test]
    fn test_file_io_read() {
        // File reading is tested on top of file writing,
        // so make sure that the file is written well before reading.

        // Initialize a file io instance.
        let file_path = std::path::Path::new("./test_read.data");
        let fio = FileIo::new(file_path);
        assert!(fio.is_ok());
        let fio = fio.unwrap();

        // Keys to write and read.
        let keys = [
            "key1",
            "key_abcd",
            "random_key",
            "key_1234",
            "a-key-that-is-very-long",
        ];

        // Write data to the file.
        for key in keys.iter() {
            write_data_and_assert!(key, fio);
        }

        // Read data from the file in random order.
        // Record the offsets of each key.
        let mut offsets = Vec::new();
        let mut offset: u64 = 0;
        for key in keys.iter() {
            offsets.push(offset);
            offset += key.len() as u64;
        }

        // Save the keys and offsets in pairs.
        let mut pairs: Vec<(&str, u64)> =
            keys.iter().zip(offsets.iter()).map(|(&s, &i)| (s, i)).collect();

        // Shuffle the key-offset pairs.
        use rand::seq::SliceRandom;
        pairs.shuffle(&mut rand::thread_rng());

        // Read data from the file in random order.
        // TODO: Test parallel reading.
        for (key, offset) in pairs.iter() {
            read_data_and_assert!(key.len(), *offset, fio);
        }

        // Clean up
        let del = std::fs::remove_file(file_path);
        assert!(del.is_ok());
    }

    #[test]
    fn test_file_io_sync() {
        let file_path = std::path::Path::new("./test_sync.data");

        // Create a file io instance
        let fio = FileIo::new(file_path);
        assert!(fio.is_ok());
        let fio = fio.unwrap();

        let key1 = "key1";
        write_data_and_assert!(key1, fio);

        let key2 = "key_abcd";
        write_data_and_assert!(key2, fio);

        // Sync the file.
        let result = fio.sync();
        assert!(result.is_ok());

        // Clean up
        let del = std::fs::remove_file(file_path);
        assert!(del.is_ok());
    }
}
