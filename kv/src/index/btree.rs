use crate::entry::EntryPos;

use std::collections::BTreeMap;

/// A wrapper over [`BTreeMap`], which is a simple in-memory BTree.
/// Inside the BTree, it stores the key and the position of the entry.
pub struct BTree {
    tree: std::sync::Arc<std::sync::RwLock<BTreeMap<Vec<u8>, EntryPos>>>,
}

impl BTree {
    /// Create a new BTree instance.
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            tree: std::sync::Arc::new(std::sync::RwLock::new(BTreeMap::new())),
        }
    }
}

impl Default for BTree {
    fn default() -> Self {
        Self::new()
    }
}

impl super::Indexer for BTree {
    fn put(&self, key: Vec<u8>, pos: EntryPos) -> bool {
        let write_guard = self.tree.write();
        let mut write_guard = match write_guard {
            Ok(guard) => guard,
            Err(_) => return false,
        };

        write_guard.insert(key, pos);
        true
    }

    fn get(&self, key: Vec<u8>) -> Option<EntryPos> {
        let read_guard = match self.tree.read() {
            Ok(guard) => guard,
            Err(_) => return None,
        };
        read_guard.get(&key).copied()
    }

    fn del(&self, key: Vec<u8>) -> bool {
        let mut write_guard = match self.tree.write() {
            Ok(guard) => guard,
            Err(_) => return false,
        };
        let remove_res = write_guard.remove(&key);
        remove_res.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::Indexer;

    #[test]
    fn test_btree_put() {
        let btree = BTree::new();

        let key1 = "".as_bytes().to_vec();
        let pos1 = EntryPos { file_id: 1, offset: 10 };
        assert!(btree.put(key1, pos1));

        let key2 = "key2".as_bytes().to_vec();
        let pos2 = EntryPos { file_id: 2, offset: 20 };
        assert!(btree.put(key2, pos2));
    }

    #[test]
    fn test_btree_get() {
        let btree = BTree::new();

        let key1 = "key1".as_bytes().to_vec();
        let pos1 = EntryPos { file_id: 1, offset: 10 };
        btree.put(key1.clone(), pos1);

        let key2 = "key2".as_bytes().to_vec();
        let pos2 = EntryPos { file_id: 2, offset: 20 };
        btree.put(key2.clone(), pos2);

        let get_res1 = btree.get(key1);
        assert!(get_res1.is_some());
        println!("{:?}", get_res1);
        let get_res1 = get_res1.unwrap();
        assert_eq!(get_res1.file_id, 1);
        assert_eq!(get_res1.offset, 10);

        let get_res2 = btree.get(key2);
        assert!(get_res2.is_some());
        println!("{:?}", get_res2);
        let get_res2 = get_res2.unwrap();
        assert_eq!(get_res2.file_id, 2);
        assert_eq!(get_res2.offset, 20);
    }

    #[test]
    fn test_btree_del() {
        let btree = BTree::new();

        let key1 = "key1".as_bytes().to_vec();
        let pos1 = EntryPos { file_id: 1, offset: 10 };
        btree.put(key1.clone(), pos1);

        let key2 = "key2".as_bytes().to_vec();
        let pos2 = EntryPos { file_id: 2, offset: 20 };
        btree.put(key2.clone(), pos2);

        let del_res1 = btree.del(key1);
        assert!(del_res1);

        let get_res2 = btree.del(key2);
        assert!(get_res2);

        let key = "not_exist".as_bytes().to_vec();
        let del_res3 = btree.del(key);
        assert!(!del_res3);
    }
}
