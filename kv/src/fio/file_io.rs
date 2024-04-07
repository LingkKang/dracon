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

    #[test]
    fn test_file_io_write() {
        let file_path = std::path::PathBuf::from("./test.data");

        // Create a file io instance
        let fio = FileIo::new(file_path.clone());
        assert!(fio.is_ok());
        let fio = fio.unwrap();

        let key1 = "key1";
        let result = fio.write(key1.as_bytes());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), key1.len());

        let key2 = "key2";
        let result = fio.write(key2.as_bytes());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), key2.len());

        // Clean up
        let del = std::fs::remove_file(file_path);
        assert!(del.is_ok());
    }

    #[test]
    fn test_file_io_read() {
        // File reading is tested on top of file writing,
        // so make sure that the file is written well before reading

        // Initialize a file io instance
        let file_path = std::path::PathBuf::from("./test.data");
        let fio = FileIo::new(file_path.clone());
        assert!(fio.is_ok());
        let fio = fio.unwrap();

        // Write data to the file
        let key1 = "key1";
        let result = fio.write(key1.as_bytes());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), key1.len());

        let key2 = "key2";
        let result = fio.write(key2.as_bytes());
        assert!(result.is_ok());
        assert_eq!(result.ok().unwrap(), key2.len());

        // Read data from the file
        let mut buf = [0u8; 4];
        let result = fio.read(&mut buf, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), key1.len());
        println!("buf: {:?}", buf);
        println!("val: {:?}", std::str::from_utf8(&buf).unwrap());

        let mut buf = [0u8; 4];
        let result = fio.read(&mut buf, key1.len());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), key2.len());
        println!("buf: {:?}", buf);
        println!("val: {:?}", std::str::from_utf8(&buf).unwrap());

        // Clean up
        let del = std::fs::remove_file(file_path);
        assert!(del.is_ok());
    }
}
