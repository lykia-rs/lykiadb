use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct IndexRecord {
    pub file_id: usize,
    pub vpos: usize,
    pub vsz: usize,
    pub tstamp: u64,
}

pub type Keys<'a> = std::collections::btree_map::Keys<'a, Vec<u8>, IndexRecord>;

/*
 * A map for mapping keys to active file, entry tuples
 * key -> | file_id | vsz | value_pos | tstamp |
*/
#[derive(Default)]
pub(crate) struct KeyDir {
    map: BTreeMap<Vec<u8>, IndexRecord>,
}

impl KeyDir {
    pub fn get(&self, key: &[u8]) -> Option<IndexRecord> {
        self.map.get(key).cloned()
    }

    pub fn put(&mut self, key: Vec<u8>, value: IndexRecord) {
        self.map.insert(key, value);
    }

    pub fn list_keys(&self) -> Keys<'_> {
        self.map.keys()
    }
}
