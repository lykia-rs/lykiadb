use std::{io, path::PathBuf};

mod store;
mod keydir;

use store::DataStore;
use keydir::KeyDir;

use crate::storage::bitcask::{
    store::DataStoreScanError,
    keydir::{IndexRecord, Keys},
};

struct Bitcask {
    data_store: DataStore,
    key_dir: KeyDir,
}

// Reference: https://riak.com/assets/bitcask-intro.pdf
impl Bitcask {
    pub fn open(dir: PathBuf) -> Result<Bitcask, DataStoreScanError> {
        let data_store = DataStore::open(dir)?;

        let instance = Bitcask {
            key_dir: data_store.to_keydir(),
            data_store,
        };

        Ok(instance)
    }

    /* Read
    - Lookup keydir for the address, then read the file
    */
    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        let index: Option<IndexRecord> = self.key_dir.get(key);

        if let Some(index) = index {
            return self.data_store.get(index);
        }

        None
    }

    /* Write
    - Append the key value pair to active data file
    - Update the keydir with the location of the newest data
    */
    pub fn put(&mut self, key: Vec<u8>, value: Vec<u8>) -> io::Result<()> {
        let index = self.data_store.put(key.clone(), value)?;
        self.key_dir.put(key, index);
        Ok(())
    }

    pub fn delete(&mut self, key: Vec<u8>) -> io::Result<()> {
        self.put(key, vec![])
    }

    pub fn list_keys(&self) -> Keys<'_> {
        self.key_dir.list_keys()
    }

    /* Merge/Compaction
    - Traverse all the old files.
    - Keep the latest keys only.
    - Output the new file
    - Append a hint file for easier read
    */
    pub fn merge() {
        // TODO
    }

    pub fn sync() {
        // TODO
    }

    pub fn close() {
        // TODO
    }
}

#[cfg(test)]
mod test {
    use crate::storage::bitcask::Bitcask;
    use itertools::Itertools;
    use std::path::PathBuf;

    #[test]
    fn open_instance() {
        let mut bc =
            Bitcask::open(PathBuf::from("/Users/monaco/workspace/lykiadb/db-data/")).unwrap();
        bc.put(
            "hello".to_string().bytes().collect_vec(),
            "world".to_string().bytes().collect_vec(),
        )
        .unwrap();
    }
}
