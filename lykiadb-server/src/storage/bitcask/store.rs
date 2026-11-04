use std::{
    fs::{self, File},
    io::{self, BufWriter, Write},
    path::PathBuf,
};

use crc64fast::Digest;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::storage::bitcask::{
    store::DataStoreScanError::IncorrectDataFileName,
    keydir::{IndexRecord, KeyDir},
};

const DATA_FILE_PREFIX: &str = "data_";
const MAX_FILE_BYTES: usize = 4096;

/*
 * Data file
 * One active append only data file

| crc | tstamp | ksz | vsz | key | value |
| --- | --------- crc coverage --------- |
 */
pub struct DataRecord {
    crc: u64,
    tstamp: u64,
    value: Vec<u8>,
    key: Vec<u8>,
}

#[derive(Clone)]
pub(crate) struct DataFile {
    id: usize,
    name: String,
    path: PathBuf,
}

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

        let last = files.last();

        let next_file = DataStore::get_next_file(dir, last);

        files.push(next_file.clone());

        Ok(DataStore {
            files,
            writer: next_file.writer()?,
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
        let mut c = Digest::new();
        c.write(key.as_slice());
        c.write(value.as_slice());

        let tstamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let result = self.writer.write(DataRecord {
            crc: c.sum64(),
            tstamp,
            key,
            value,
        })?;

        Ok(result.index_record)
    }

    pub fn to_keydir(&self) -> KeyDir {
        let keydir = KeyDir::default();
        // TODO(vck): implement keydir build
        keydir
    }
}

impl DataFile {
    pub fn writer(&self) -> io::Result<DataFileWriter> {
        let path = self.path.clone();
        let file = File::create(path)?;
        let writer = BufWriter::new(file);

        Ok(DataFileWriter {
            file: self.clone(),
            writer,
            written_bytes: 0,
            max_file_bytes: MAX_FILE_BYTES,
        })
    }

    pub fn get(&self, _record: IndexRecord) -> Vec<u8> {
        // TODO(vck): Implement get
        vec![]
    }
}

pub(crate) struct DataFileWriter {
    file: DataFile,
    writer: BufWriter<File>,
    written_bytes: usize,
    max_file_bytes: usize,
}

struct WriteResult {
    switch_files: bool,
    index_record: IndexRecord,
}

impl DataFileWriter {
    pub fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }

    pub fn write(&mut self, record: DataRecord) -> io::Result<WriteResult> {
        let ksz = record.key.len();
        let vsz = record.value.len();
        let pos = self.written_bytes;
        let mut written = ksz + vsz;

        written += self.writer.write(&record.crc.to_le_bytes())?;
        written += self.writer.write(&record.tstamp.to_le_bytes())?;
        written += self.writer.write(&ksz.to_le_bytes())?;
        written += self.writer.write(&vsz.to_le_bytes())?;
        self.writer.write_all(&record.key)?;
        self.writer.write_all(&record.value)?;
        self.written_bytes += written;

        // true: it is time to switch to a new file
        let switch_files = self.written_bytes >= self.max_file_bytes;

        if switch_files {
            self.flush();
        }

        Ok(WriteResult {
            switch_files,
            index_record: IndexRecord {
                file_id: self.file.id,
                vpos: pos,
                vsz,
                tstamp: record.tstamp,
            },
        })
    }
}
