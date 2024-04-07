use crate::err::Result;

pub mod file_io;

/// The trait for managing I/O operations.
/// At the moment, it only support std file I/O.
pub trait IoManager: Sync + Send {
    /// Read data from designated position.
    fn read(&self, buf: &mut [u8], offset: u64) -> Result<u64>;

    /// Write data to designated position.
    fn write(&self, buf: &[u8]) -> Result<u64>;

    /// Sync (persist) data to disk.
    fn sync(&self) -> Result<()>;
}
