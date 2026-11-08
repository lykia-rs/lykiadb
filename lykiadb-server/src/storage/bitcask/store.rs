use std::{
    fs::{self},
    io::{self},
    path::PathBuf,
};

use crate::storage::bitcask::{
    file::{DataFile, DataFileReadError, DataFileWriteError, DataRecord},
    keydir::{IndexRecord, KeyDir},
    store::DataStoreScanError::IncorrectDataFileName,
};

const DATA_FILE_PREFIX: &str = "data_";
const MAX_FILE_BYTES: usize = 1 << 20;

pub(crate) struct DataStore {
    files: Vec<DataFile>,
    max_file_bytes: usize,
    dir: PathBuf,
}

#[derive(Debug)]
pub enum DataStoreScanError {
    IoError(std::io::Error),
    IncorrectDataFileName(String),
}

impl From<std::io::Error> for DataStoreScanError {
    fn from(err: std::io::Error) -> Self {
        DataStoreScanError::IoError(err)
    }
}

impl DataStore {
    pub fn open(dir: PathBuf) -> Result<DataStore, DataStoreScanError> {
        let mut files: Vec<DataFile> = vec![];

        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let file_name = entry.file_name();

            if let Some(file_name) = file_name.to_str()
                && let Some(id_str) = file_name.strip_prefix(DATA_FILE_PREFIX)
            {
                match id_str.parse::<usize>() {
                    Ok(id) => {
                        files.push(DataFile::new(id, entry.path(), file_name.to_string(), true))
                    }
                    Err(_) => return Err(IncorrectDataFileName(id_str.to_string())),
                }
            }
        }

        files.sort_by_key(|a| a.get_id());

        files.push(DataStore::get_next_file(&dir, files.last())?);

        Ok(DataStore {
            files,
            dir,
            max_file_bytes: MAX_FILE_BYTES,
        })
    }

    fn get_next_file(dir: &PathBuf, last_file: Option<&DataFile>) -> io::Result<DataFile> {
        let id = match last_file {
            Some(file) => file.get_id() + 1,
            None => 0,
        };

        let mut name = DATA_FILE_PREFIX.to_string();
        name.push_str(&format!("{:04}", id));

        let mut file = DataFile::new(id, dir.clone(), name, false);

        file.open()?;

        Ok(file)
    }

    fn switch_to_next_file(&mut self) -> io::Result<()> {
        let current_file = self.files.last_mut();

        current_file.unwrap().archive()?;

        self.files
            .push(DataStore::get_next_file(&self.dir, self.files.last())?);

        Ok(())
    }

    pub fn get(&mut self, record: IndexRecord) -> Result<Option<Vec<u8>>, DataFileReadError> {
        let data_file = self.files.get_mut(record.file_id);

        data_file.map(|data_file| data_file.get(record)).transpose()
    }

    pub fn put(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<IndexRecord, DataFileWriteError> {
        let current_file = self.files.last_mut().unwrap();
        let index_record = current_file.write(DataRecord::new(key, value))?;

        // true: it is time to switch to a new file
        if current_file.get_written_bytes() >= self.max_file_bytes {
            self.switch_to_next_file()?;
        }

        Ok(index_record)
    }

    pub fn build_keydir(&self) -> KeyDir {
        let keydir = KeyDir::default();
        // TODO(vck): implement keydir build
        keydir
    }
}
