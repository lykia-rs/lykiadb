use std::{fs::File, io::{self, BufWriter, Write}, path::PathBuf, time::{SystemTime, UNIX_EPOCH}};

use crc64fast::Digest;

use crate::storage::bitcask::keydir::IndexRecord;

const MAX_FILE_BYTES: usize = 4096;

/*
 * Data file
 * One active append only data file

| crc | tstamp | ksz | vsz | key | value |
| --- | --------- crc coverage --------- |
 */
pub(crate) struct DataRecord {
    key: Vec<u8>,
    value: Vec<u8>,
    tstamp: u64
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
    pub path: PathBuf,
}

pub(crate) struct DataFileWriter {
    file: DataFile,
    writer: BufWriter<File>,
    written_bytes: usize,
    max_file_bytes: usize,
}

pub(crate) struct WriteResult {
    pub switch_files: bool,
    pub index_record: IndexRecord,
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

impl DataFileWriter {
    pub fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }

    pub fn write(&mut self, record: DataRecord) -> io::Result<WriteResult> {
        let pos = self.written_bytes;

        let written = record.write_to(&mut self.writer)?;
        
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
                vsz: record.value.len(),
                tstamp: record.tstamp,
            },
        })
    }
}
