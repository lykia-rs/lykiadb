use std::{fs::File, io::{self, BufWriter, Write}, path::PathBuf};

use crate::storage::bitcask::keydir::IndexRecord;

const MAX_FILE_BYTES: usize = 4096;

/*
 * Data file
 * One active append only data file

| crc | tstamp | ksz | vsz | key | value |
| --- | --------- crc coverage --------- |
 */
pub(crate) struct DataRecord {
    pub crc: u64,
    pub tstamp: u64,
    pub value: Vec<u8>,
    pub key: Vec<u8>,
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
