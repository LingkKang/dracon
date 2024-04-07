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
        let mut write_guard = self.file.write().unwrap();

        write_guard
            .seek(std::io::SeekFrom::Start(offset as u64))
            .map_err(|e| {
                log::error!("Seek in data file err: {}", e);
                Err { code: ErrCode::SeekInDataFileFailed, msg: e.to_string() }
            })
            .unwrap();

        let read_bytes = write_guard
            .read(buf)
            .map_err(|e| {
                log::error!("Read data file err: {}", e);
                Err { code: ErrCode::ReadDataFileFailed, msg: e.to_string() }
            })
            .unwrap();

        Ok(read_bytes)
    }

    fn write(&self, buf: &[u8]) -> Result<usize> {
        let write_guard = self.file.write();
        let mut write_guard = match write_guard {
            Ok(guard) => guard,
            Err(e) => {
                log::error!("Failed to write file: {}", e);
                return Err(Err {
                    code: ErrCode::WriteDataFileFailed,
                    msg: e.to_string(),
                });
            }
        };
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
        let key1 = "key1";
        let key2 = "key_abcd";

        // Write data to the file.
        write_data_and_assert!(key1, fio);
        write_data_and_assert!(key2, fio);

        // Read data from the file.
        read_data_and_assert!(key1.len(), 0, fio);
        read_data_and_assert!(key2.len(), key1.len(), fio);

        // Clean up
        let del = std::fs::remove_file(file_path);
        assert!(del.is_ok());
    }
}
