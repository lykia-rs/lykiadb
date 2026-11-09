use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Seek, Write},
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
    readonly: bool,
}

#[derive(Debug)]
pub enum DataFileReadError {
    IoError(std::io::Error),
    FileNotOpened,
}

impl From<std::io::Error> for DataFileReadError {
    fn from(err: std::io::Error) -> Self {
        DataFileReadError::IoError(err)
    }
}

#[derive(Debug)]
pub enum DataFileWriteError {
    IoError(std::io::Error),
    FileNotOpened,
    WriteAttemptToReadonlyFile,
}

impl From<std::io::Error> for DataFileWriteError {
    fn from(err: std::io::Error) -> Self {
        DataFileWriteError::IoError(err)
    }
}

impl DataFile {
    pub fn new(id: usize, dir: PathBuf, name: String, readonly: bool) -> DataFile {
        DataFile {
            id,
            dir,
            name,
            handle: None,
            written_bytes: 0,
            readonly,
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
                .read(true)
                .create(!self.readonly)
                .append(!self.readonly)
                .open(path)?,
        );

        Ok(())
    }

    pub fn archive(&mut self) -> io::Result<()> {
        self.close()?;
        self.readonly = true;
        self.written_bytes = 0;
        self.open()?;
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

    pub fn get(&mut self, record: &IndexRecord) -> Result<Vec<u8>, DataFileReadError> {
        if let Some(file) = &mut self.handle {
            let mut value = vec![0; record.vsz];
            file.seek(io::SeekFrom::Start(record.vpos as u64))?;
            file.read_exact(&mut value)?;
            return Ok(value);
        }
        Err(DataFileReadError::FileNotOpened)
    }

    pub fn write(&mut self, record: DataRecord) -> Result<IndexRecord, DataFileWriteError> {
        if self.readonly {
            return Err(DataFileWriteError::WriteAttemptToReadonlyFile);
        }

        if let Some(file) = &mut self.handle {
            self.written_bytes += record.write_to(file)?;

            return Ok(IndexRecord {
                file_id: self.id,
                vpos: self.written_bytes - record.value.len(),
                vsz: record.value.len(),
                tstamp: record.tstamp,
            });
        }

        Err(DataFileWriteError::FileNotOpened)
    }
}

#[cfg(test)]
mod tests {
    use crate::storage::bitcask::{file::{DataFile, DataFileWriteError, DataRecord}, keydir::IndexRecord};
    use std::{assert_matches, path::PathBuf};
    use tempfile::tempdir;

    #[test]
    fn test_simple_write() {
        let tmp_dir = tempdir().unwrap();
        let mut file = DataFile::new(
            1,
            PathBuf::from(tmp_dir.path()),
            "data_0001".to_owned(),
            false,
        );
        file.open().unwrap();
        let index_record = file
            .write(DataRecord::new(b"hello".to_vec(), b"world".to_vec()))
            .unwrap();

        assert_eq!(file.get_written_bytes(), 42);
        assert_eq!(index_record.file_id, 1);
        assert_eq!(index_record.vpos, 37);
        assert_eq!(index_record.vsz, 5);
        tmp_dir.close().unwrap();
    }

    #[test]
    fn test_read_after_archive() {
        let tmp_dir = tempdir().unwrap();
        let mut file = DataFile::new(
            1,
            PathBuf::from(tmp_dir.path()),
            "data_0001".to_owned(),
            false,
        );
        file.open().unwrap();

        let r0 = file
            .write(DataRecord::new(b"hello".to_vec(), b"world".to_vec()))
            .unwrap();

        assert_eq!(file.get_written_bytes(), 42);
        assert_eq!(r0.vpos, 37);
        assert_eq!(r0.vsz, 5);

        assert_eq!(file.readonly, false);
        file.archive().unwrap();
        assert_eq!(file.readonly, true);

        assert_matches!(
            file.write(DataRecord::new(b"hello".to_vec(), b"world".to_vec()))
                .unwrap_err(),
            DataFileWriteError::WriteAttemptToReadonlyFile
        );

        assert_eq!(file.get(&r0).unwrap(), b"world".to_vec());

        tmp_dir.close().unwrap();
    }

    #[test]
    fn test_interleaved_reads_writes() {
        let tmp_dir = tempdir().unwrap();
        let mut file = DataFile::new(
            1,
            PathBuf::from(tmp_dir.path()),
            "data_0001".to_owned(),
            false,
        );

        // Open
        file.open().unwrap();

        // First record
        let r0 = file
            .write(DataRecord::new(b"hello".to_vec(), b"world".to_vec()))
            .unwrap();

        assert_eq!(file.get_written_bytes(), 42);
        assert_eq!(r0.vpos, 37);
        assert_eq!(r0.vsz, 5);

        // Read 1st
        assert_eq!(file.get(&r0).unwrap(), b"world".to_vec());

        // Second record
        let r1 = file
            .write(DataRecord::new(b"hello".to_vec(), b"".to_vec()))
            .unwrap();

        assert_eq!(file.get_written_bytes(), 79);
        assert_eq!(r1.vpos, 79);
        assert_eq!(r1.vsz, 0);

        // Read 1st and 2nd
        assert_eq!(file.get(&r0).unwrap(), b"world".to_vec());
        assert_eq!(file.get(&r1).unwrap(), b"".to_vec());

        // Third record
        let r2 = file
            .write(DataRecord::new(b"key".to_vec(), b"val".to_vec()))
            .unwrap();

        assert_eq!(file.get_written_bytes(), 117);
        assert_eq!(r2.vpos, 114);
        assert_eq!(r2.vsz, 3);

        // Read all
        assert_eq!(file.get(&r0).unwrap(), b"world".to_vec());
        assert_eq!(file.get(&r1).unwrap(), b"".to_vec());
        assert_eq!(file.get(&r2).unwrap(), b"val".to_vec());

        // Close
        tmp_dir.close().unwrap();
    }
}
