//! This module defines the API of indexing data in memory.

pub mod btree;

/// The API of indexing data in memory.
pub trait Indexer {
    /// Put a key into the position.
    fn put(&self, key: Vec<u8>, pos: crate::entry::EntryPos) -> bool;

    /// Get the position of a key.
    fn get(&self, key: Vec<u8>) -> Option<crate::entry::EntryPos>;

    /// Delete a key.
    fn del(&self, key: Vec<u8>) -> bool;
}
