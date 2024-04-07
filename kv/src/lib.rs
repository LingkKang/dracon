//! The key-value storage engine implementation.
//!
//! See [Bitcask: A Log-Structured Hash Table for Fast Key/Value Data](
//! https://riak.com/assets/bitcask-intro.pdf)
//! by **Justin Sheehy** and **David Smith** for more details.

mod entry;
mod err;
mod fio;
mod index;

pub mod db;
