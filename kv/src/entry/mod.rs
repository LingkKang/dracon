//! In the bitcask storage model, the key-value store
//! is a collection of files, each of which is a log
//! of key-value pairs. The key-value pairs are stored
//! in the order they are written to the file.

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
