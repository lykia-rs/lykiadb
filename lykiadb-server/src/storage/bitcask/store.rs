use std::{
    fs::{self},
    io::{self},
    path::PathBuf,
};


use crate::storage::bitcask::{
    file::{DataFile, DataFileWriter, DataRecord}, keydir::{IndexRecord, KeyDir}, store::DataStoreScanError::IncorrectDataFileName,
};

const DATA_FILE_PREFIX: &str = "data_";

pub(crate) struct DataStore {
    files: Vec<DataFile>,
    writer: DataFileWriter,
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
                    Ok(id) => files.push(DataFile {
                        id,
                        path: entry.path(),
                        name: file_name.to_string(),
                    }),
                    Err(_) => return Err(IncorrectDataFileName(id_str.to_string())),
                }
            }
        }

        files.sort_by_key(|a| a.id);

        let next_file = DataStore::get_next_file(dir, files.last());

        let writer = next_file.writer()?;

        files.push(next_file);

        Ok(DataStore {
            files,
            writer
        })
    }

    fn get_next_file(dir: PathBuf, last_file: Option<&DataFile>) -> DataFile {
        let id = match last_file {
            Some(file) => file.id + 1,
            None => 1,
        };
        let mut path = dir.clone();
        let mut name = DATA_FILE_PREFIX.to_string();

        name.push_str(&format!("{:04}", id));
        path.push(&name);

        DataFile { id, path, name }
    }

    pub fn get(&self, record: IndexRecord) -> Option<Vec<u8>> {
        let data_file = self.files.get(record.file_id);
        data_file.map(|data_file| data_file.get(record))
    }

    pub fn put(&mut self, key: Vec<u8>, value: Vec<u8>) -> io::Result<IndexRecord> {
        let result = self.writer.write(DataRecord::new(
            key,
            value
        ))?;

        Ok(result.index_record)
    }

    pub fn to_keydir(&self) -> KeyDir {
        let keydir = KeyDir::default();
        // TODO(vck): implement keydir build
        keydir
    }
}
