use super::IoManager;
use crate::err::Err;
use crate::err::ErrCode;
use crate::err::Result;

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
        todo!()
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

        let key1 = "key1\n";
        let result = fio.write(key1.as_bytes());
        assert!(result.is_ok());
        assert!(result.unwrap() == key1.len());

        let key2 = "key2\n";
        let result = fio.write(key2.as_bytes());
        assert!(result.is_ok());
        assert!(result.unwrap() == key2.len());

        // Clean up
        let del = std::fs::remove_file(file_path);
        assert!(del.is_ok());
    }
}
