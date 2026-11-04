use std::{io, path::PathBuf};

mod datadir;
mod keydir;

use datadir::DataDir;
use keydir::KeyDir;

use crate::storage::bitcask::{
    datadir::DataDirScanError,
    keydir::{IndexRecord, Keys},
};

struct Bitcask {
    data_dir: DataDir,
    key_dir: KeyDir,
}

// Reference: https://riak.com/assets/bitcask-intro.pdf
impl Bitcask {
    pub fn open(dir: PathBuf) -> Result<Bitcask, DataDirScanError> {
        let data_dir = DataDir::open(dir)?;

        let instance = Bitcask {
            key_dir: data_dir.to_keydir(),
            data_dir,
        };

        Ok(instance)
    }

    /* Read
    - Lookup keydir for the address, then read the file
    */
    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        let index: Option<IndexRecord> = self.key_dir.get(key);

        if let Some(index) = index {
            return self.data_dir.get(index);
        }

        None
    }

    /* Write
    - Append the key value pair to active data file
    - Update the keydir with the location of the newest data
    */
    pub fn put(&mut self, key: Vec<u8>, value: Vec<u8>) -> io::Result<()> {
        let index = self.data_dir.put(key.clone(), value)?;
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
