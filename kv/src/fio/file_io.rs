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
    pub fn new(file_name: std::path::PathBuf) -> Result<Self> {
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
    #[allow(unused_variables)]
    fn read(&self, buf: &mut [u8], offset: usize) -> Result<usize> {
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
            .seek(std::io::SeekFrom::Start(offset as u64))
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

        Ok(read_bytes)
    }

    fn write(&self, buf: &[u8]) -> Result<usize> {
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
            Ok(size) => Ok(size),
            Err(e) => {
                log::error!("Failed to write file: {}", e);
                Err(Err {
                    code: ErrCode::WriteDataFileFailed,
                    msg: e.to_string(),
                })
            }
        }
    }

    fn sync(&self) -> Result<()> {
        todo!()
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
            assert_eq!(result.unwrap(), $data.len());
        };
    }

    #[test]
    fn test_file_io_write() {
        let file_path = std::path::PathBuf::from("./test_write.data");

        // Create a file io instance
        let fio = FileIo::new(file_path.clone());
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
            assert_eq!(result.unwrap(), $len);
            // Print the read data in decimal.
            println!("buf: {:?}", buf);
            // Print the read data in string.
            println!("val: {:?}", std::str::from_utf8(&buf).unwrap());
        };
    }

    #[test]
    fn test_file_io_read() {
        // File reading is tested on top of file writing,
        // so make sure that the file is written well before reading.

        // Initialize a file io instance.
        let file_path = std::path::PathBuf::from("./test_read.data");
        let fio = FileIo::new(file_path.clone());
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
        let mut offset: usize = 0;
        for key in keys.iter() {
            offsets.push(offset);
            offset += key.len();
        }

        // Save the keys and offsets in pairs.
        let mut pairs: Vec<(&str, usize)> =
            keys.iter().zip(offsets.iter()).map(|(&s, &i)| (s, i)).collect();

        // Shuffle the key-offset pairs.
        use rand::seq::SliceRandom;
        pairs.shuffle(&mut rand::thread_rng());

        // Read data from the file in random order.
        for (key, offset) in pairs.iter() {
            read_data_and_assert!(key.len(), *offset, fio);
        }

        // Clean up
        let del = std::fs::remove_file(file_path);
        assert!(del.is_ok());
    }
}
