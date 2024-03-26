//! This module defines the API of indexing data in memory.

pub mod btree;

use crate::entry::EntryPos;

/// The API of indexing data in memory.
pub trait Indexer {
    /// Put a key into the position.
    fn put(&self, key: Vec<u8>, pos: EntryPos) -> bool;

    /// Get the position of a key.
    fn get(&self, key: Vec<u8>) -> Option<EntryPos>;

    /// Delete a key.
    fn del(&self, key: Vec<u8>) -> bool;
}
