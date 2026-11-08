use std::path::PathBuf;

mod file;
mod keydir;
mod store;

use keydir::KeyDir;
use store::DataStore;

use crate::storage::bitcask::{
    file::{DataFileReadError, DataFileWriteError},
    keydir::Keys,
    store::DataStoreScanError,
};

struct Bitcask {
    data_store: DataStore,
    key_dir: KeyDir,
}

// Reference: https://riak.com/assets/bitcask-intro.pdf
impl Bitcask {
    pub fn open(dir: PathBuf) -> Result<Bitcask, DataStoreScanError> {
        let data_store = DataStore::open(dir)?;

        Ok(Bitcask {
            key_dir: data_store.build_keydir(),
            data_store,
        })
    }

    /* Read
    - Lookup keydir for the address, then read the file
    */
    pub fn get(&mut self, key: &[u8]) -> Result<Option<Vec<u8>>, DataFileReadError> {
        match self
            .key_dir
            .get(key)
            .map(|index| self.data_store.get(index))
        {
            Some(x) => x,
            None => Ok(None),
        }
    }

    /* Write
    - Append the key value pair to active data file
    - Update the keydir with the location of the newest data
    */
    pub fn put(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<(), DataFileWriteError> {
        let index = self.data_store.put(key.clone(), value)?;
        self.key_dir.put(key, index);
        Ok(())
    }

    pub fn delete(&mut self, key: Vec<u8>) -> Result<(), DataFileWriteError> {
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
    use std::path::PathBuf;

    use tempfile::tempdir;

    use crate::storage::bitcask::Bitcask;

    #[test]
    fn open_instance() {
        let tmp_dir = tempdir().unwrap();
        let mut bc = Bitcask::open(PathBuf::from(tmp_dir.path())).unwrap();

        bc.put(b"hello".to_vec(), b"world".to_vec()).unwrap();

        assert_eq!(
            bc.get(b"hello".to_vec().as_slice()).unwrap(),
            Some(b"world".to_vec())
        );

        tmp_dir.close().unwrap();
    }
}
