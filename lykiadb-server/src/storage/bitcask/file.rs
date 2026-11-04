use std::{
    fs::File,
    io::{self, BufWriter, Write},
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

    pub fn write_to<T: Write>(&self, buf: &mut BufWriter<T>) -> io::Result<usize> {
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

#[derive(Clone)]
pub(crate) struct DataFile {
    pub id: usize,
    pub name: String,
    pub dir: PathBuf,
}

pub(crate) struct DataFileWriter {
    file: DataFile,
    writer: BufWriter<File>,
    total_bytes: usize,
}

impl DataFile {
    pub fn writer(&self) -> io::Result<DataFileWriter> {
        let mut path = self.dir.clone();
        path.push(&self.name);
        let file = File::create(path)?;
        let writer = BufWriter::new(file);

        Ok(DataFileWriter {
            file: self.clone(),
            writer,
            total_bytes: 0,
        })
    }

    pub fn get(&self, _record: IndexRecord) -> Vec<u8> {
        // TODO(vck): Implement get
        vec![]
    }
}

impl DataFileWriter {
    pub fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }

    pub fn get_total_bytes(&self) -> usize {
        self.total_bytes
    }

    pub fn write(&mut self, record: DataRecord) -> io::Result<IndexRecord> {
        let pos = self.total_bytes;

        let written = record.write_to(&mut self.writer)?;

        self.total_bytes += written;

        Ok(IndexRecord {
            file_id: self.file.id,
            vpos: pos,
            vsz: record.value.len(),
            tstamp: record.tstamp,
        })
    }
}
