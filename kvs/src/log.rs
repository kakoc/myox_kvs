use serde::{Deserialize, Serialize};

use std::io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write};

use crate::error::Error;

const LOG_FILE_EXTENSION_NAME: &'static str = "log";
const DEFAULT_LOG_NAME: &'static str = "1.log";

enum LogError {}

#[derive(Deserialize, Serialize, Debug)]
pub enum LogCommand {
    Insert { key: String, value: String },
    Remove { key: String },
}

pub fn create_log_file(
    existed_log_files_names: &Vec<String>,
    path: &std::path::Path,
) -> Result<(std::fs::File, String), Error> {
    let last_i = existed_log_files_names
        .last()
        .unwrap_or(&DEFAULT_LOG_NAME.to_string())
        .trim_end_matches(".log")
        .parse::<usize>()
        .expect("error while trying to parse log's name number part");

    let log_name = format!("{}.{}", last_i + 1, LOG_FILE_EXTENSION_NAME);
    let file = std::fs::File::create(path.join(&log_name))?;

    Ok((file, log_name))
}

#[derive(Debug)]
pub struct LogReader<T: Read + Seek> {
    reader: std::io::BufReader<T>,
    pos: u64,
}

impl<T: Read + Seek> LogReader<T> {
    // -------------------------------------------Log error
    pub fn new(mut reader: T) -> io::Result<Self> {
        let pos = reader.seek(SeekFrom::Current(0))?;
        let reader = BufReader::new(reader);

        Ok(LogReader { reader, pos })
    }
}

impl<T: Read + Seek> Read for LogReader<T> {
    // -------------------------------------------Log error
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let read_bytes = self.reader.read(buf)?;
        self.pos += read_bytes as u64;

        Ok(read_bytes)
    }
}

impl<T: Read + Seek> Seek for LogReader<T> {
    // -------------------------------------------Log error
    fn seek(&mut self, pos: std::io::SeekFrom) -> io::Result<u64> {
        self.pos = self.reader.seek(pos)?;

        Ok(self.pos)
    }
}

#[derive(Debug)]
pub struct LogWriter<T: Write + Seek> {
    writer: std::io::BufWriter<T>,
    pub pos: u64,
}

impl<T: Write + Seek> LogWriter<T> {
    pub fn new(mut writer: T) -> io::Result<Self> {
        let pos = writer.seek(SeekFrom::Start(0))?;
        let writer = BufWriter::new(writer);

        Ok(LogWriter { writer, pos })
    }
}

impl<T: Write + Seek> Write for LogWriter<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let written_len_bytes = self.writer.write(buf)?;
        self.pos += written_len_bytes as u64;

        Ok(written_len_bytes)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

impl<T: Write + Seek> Seek for LogWriter<T> {
    fn seek(&mut self, seek_len: SeekFrom) -> io::Result<u64> {
        let curr_pos = self.writer.seek(seek_len)?;
        self.pos = curr_pos as u64;

        Ok(self.pos)
    }
}
