use std::{
    fs::{File, OpenOptions},
    io::{self, Write},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use crc64fast::Digest;

use crate::storage::bitcask::keydir::IndexRecord;

/*
 * Data file
 * One active append only data file

| crc | tstamp | ksz | vsz | key | value |
| --- | --------- crc coverage --------- |
 */
pub(crate) struct DataRecord {
    key: Vec<u8>,
    value: Vec<u8>,
    tstamp: u64,
}

impl DataRecord {
    pub fn new(key: Vec<u8>, value: Vec<u8>) -> DataRecord {
        let tstamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        DataRecord { key, value, tstamp }
    }

    pub fn write_to<T: Write>(&self, buf: &mut T) -> io::Result<usize> {
        let ksz = self.key.len();
        let vsz = self.value.len();

        let mut c = Digest::new();
        c.write(&self.tstamp.to_le_bytes());
        c.write(&ksz.to_le_bytes());
        c.write(&vsz.to_le_bytes());
        c.write(self.key.as_slice());
        c.write(self.value.as_slice());
        let crc = c.sum64();

        let mut written = ksz + vsz;
        written += buf.write(&crc.to_le_bytes())?;
        written += buf.write(&self.tstamp.to_be_bytes())?;
        written += buf.write(&ksz.to_le_bytes())?;
        written += buf.write(&vsz.to_le_bytes())?;
        buf.write_all(&self.key)?;
        buf.write_all(&self.value)?;

        Ok(written)
    }
}

pub(crate) struct DataFile {
    id: usize,
    dir: PathBuf,
    name: String,
    handle: Option<File>,
    written_bytes: usize,
}

#[derive(Debug)]
pub enum DataFileWriteError {
    IoError(std::io::Error),
    WriteAttemptToClosedFile,
}

impl From<std::io::Error> for DataFileWriteError {
    fn from(err: std::io::Error) -> Self {
        DataFileWriteError::IoError(err)
    }
}

impl DataFile {
    pub fn new(id: usize, dir: PathBuf, name: String) -> DataFile {
        DataFile {
            id,
            dir,
            name,
            handle: None,
            written_bytes: 0,
        }
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn open(&mut self) -> io::Result<()> {
        let mut path = self.dir.clone();
        path.push(&self.name);

        self.handle = Some(
            OpenOptions::new()
                .create(true)
                .read(true)
                .append(true)
                .open(path)?,
        );

        Ok(())
    }

    pub fn close(&mut self) -> io::Result<()> {
        self.flush()?;
        self.handle = None;
        Ok(())
    }

    pub fn flush(&mut self) -> io::Result<()> {
        if let Some(file) = &mut self.handle {
            file.flush()?;
        }
        Ok(())
    }

    pub fn get_written_bytes(&self) -> usize {
        self.written_bytes
    }

    pub fn get(&self, _record: IndexRecord) -> Vec<u8> {
        // TODO(vck): Implement get
        vec![]
    }

    pub fn write(&mut self, record: DataRecord) -> Result<IndexRecord, DataFileWriteError> {
        if let Some(file) = &mut self.handle {
            let pos = self.written_bytes;

            self.written_bytes += record.write_to(file)?;

            return Ok(IndexRecord {
                file_id: self.id,
                vpos: pos,
                vsz: record.value.len(),
                tstamp: record.tstamp,
            });
        }

        Err(DataFileWriteError::WriteAttemptToClosedFile)
    }
}
