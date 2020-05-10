use serde_json::Deserializer;

use std::collections::HashMap;
use std::ffi::OsString;
use std::fs::{read_dir, File, ReadDir};
use std::io::{copy, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use crate::error::Error;
use crate::{
    log::{create_log_file, LogCommand, LogReader, LogWriter},
    KvsEngine,
};

pub type Result<T> = std::result::Result<T, Error>;

const COMPACTION_THRESHOLD: usize = 1024 * 1024;

type LogName = String;
type PositionInLog = usize;
type LenInLog = usize;
pub type KvStoreValue = (LogName, PositionInLog, LenInLog);

// #[derive(Default)]
/// key-value storage model
#[derive(Debug)]
pub struct KvStore {
    store: HashMap<String, KvStoreValue>,
    readers: HashMap<String, LogReader<std::fs::File>>,
    writer: LogWriter<std::fs::File>,
    session_log_name: String,
    path: PathBuf,
    uncompacted: usize,
}

impl KvsEngine for KvStore {
    /// get value by key
    fn get(&mut self, key: String) -> Result<Option<String>> {
        if let Some((log_name, position_in_log, len_in_log)) = self.store.get(&key) {
            if let Some(reader) = self.readers.get_mut(log_name) {
                reader.seek(SeekFrom::Start(*position_in_log as u64));
                if let Some(LogCommand::Insert { value, .. }) =
                    serde_json::from_reader(reader.take(*len_in_log as u64))?
                {
                    return Ok(Some(value));
                } else {
                    return Err(Error::KeyNotFound);
                }
            } else {
                return Err(Error::LogReaderNotFound);
            }
        } else {
            return Ok(None);
        };

        return Ok(None);
    }

    /// insert value at key
    fn set(&mut self, key: String, value: String) -> Result<()> {
        let command = LogCommand::Insert {
            key: key.clone(),
            value,
        };
        let mut pos = self.writer.pos;
        serde_json::to_writer(&mut self.writer, &command)?;
        self.writer.flush()?;

        if let Some((_, _, len)) = self.store.insert(
            key.clone(),
            (
                self.session_log_name.to_owned(),
                pos as usize,
                (self.writer.pos - pos) as usize,
            ),
        ) {
            self.uncompacted += len;
        }

        if self.uncompacted > COMPACTION_THRESHOLD {
            self.compact();
        }

        Ok(())
    }

    /// remove value at key
    fn remove(&mut self, key: String) -> Result<()> {
        if let None = self.store.get(&key) {
            return Err(Error::KeyNotFound);
        }

        let command = LogCommand::Remove { key: key.clone() };
        serde_json::to_writer(&mut self.writer, &command)?;
        self.writer.flush()?;

        self.store.remove(&key);

        Ok(())
    }
}

impl KvStore {
    /// default factory
    pub fn new(
        session_log_writer: LogWriter<std::fs::File>,
        session_log_name: String,
        path: PathBuf,
    ) -> Self {
        KvStore {
            store: HashMap::new(),
            readers: HashMap::new(),
            writer: session_log_writer,
            session_log_name,
            path,
            uncompacted: 0,
        }
    }

    pub fn open(path: &Path) -> Result<Self> {
        std::fs::create_dir_all(&path)?;

        let log_files_names = KvStore::get_log_files_names_by_path(read_dir(path)?)?;
        let (session_log_file, log_file_name) = create_log_file(&log_files_names, path)?;

        let session_log_writer = LogWriter::new(
            std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .read(true)
                .append(true)
                .open(&path.join(log_file_name.clone()))?,
        )?;
        let mut store = Self::new(session_log_writer, log_file_name.clone(), path.into());

        store.uncompacted = store.populate_store_from_log_files(&path, log_files_names)?;
        store.readers.insert(
            log_file_name.clone(),
            LogReader::new(std::fs::File::open(path.join(&log_file_name))?)?,
        );

        Ok(store)
    }

    fn populate_store_from_log_files(
        &mut self,
        base_path: &Path,
        log_files_names: Vec<String>,
    ) -> Result<usize> {
        let mut uncompacted = 0;
        for file_name in log_files_names.iter() {
            let file = File::open(&base_path.join(file_name))?;
            let s = std::fs::read_to_string(&base_path.join(file_name))?;

            let start_pos = 0;
            let mut reader = LogReader::new(file)?;
            let mut stream = Deserializer::from_reader(&mut reader).into_iter::<LogCommand>();

            let mut pos: usize = start_pos as usize;
            while let Some(Ok(command)) = stream.next() {
                let curr_pos = stream.byte_offset();

                if let Some((_, _, len)) =
                    self.exec_command(&command, file_name.to_owned(), pos, curr_pos - pos)
                {
                    uncompacted += len;
                    if let &LogCommand::Remove { .. } = &command {
                        uncompacted += curr_pos - pos;
                    }
                }
                pos = curr_pos;
            }

            self.readers.insert(file_name.to_owned(), reader);
        }

        Ok(uncompacted)
    }

    fn exec_command(
        &mut self,
        command: &LogCommand,
        log_name: String,
        start: usize,
        len: usize,
    ) -> Option<KvStoreValue> {
        match command {
            LogCommand::Insert { key, .. } => {
                self.store.insert(key.to_owned(), (log_name, start, len))
            }
            LogCommand::Remove { key, .. } => self.store.remove(&key.to_owned()),
        }
    }

    fn get_log_files_names_by_path(dir: ReadDir) -> Result<Vec<String>> {
        let mut files: Vec<String> = dir
            .flat_map(|res| -> Result<_> { Ok(res?.path()) })
            .filter(|path| path.is_file() && path.extension() == Some("log".as_ref()))
            .flat_map(|file| OsString::into_string(file.file_name().unwrap().into()).ok())
            .filter(|file| match file.rfind(".log") {
                Some(i) if i + 3 == file.len() - 1 => true,
                _ => false,
            })
            .collect();

        files.sort_unstable();

        Ok(files)
    }

    fn compact(&mut self) -> Result<()> {
        let curr_gen = self
            .session_log_name
            .trim_end_matches(".log")
            .parse::<usize>()?;

        let new_gen = curr_gen + 2;
        let comp_gen = curr_gen + 1;

        let mut comp_writer = LogWriter::new(
            std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .read(true)
                .append(true)
                .open(self.path.join(format!("{}.{}", comp_gen, "log")))?,
        )?;
        self.writer = LogWriter::new(
            std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .read(true)
                .append(true)
                .open(self.path.join(format!("{}.{}", new_gen, "log")))?,
        )?;
        self.readers.insert(
            format!("{}.{}", comp_gen, "log"),
            LogReader::new(std::fs::File::open(
                self.path.join(format!("{}.{}", comp_gen, "log")),
            )?)?,
        );
        self.readers.insert(
            format!("{}.{}", new_gen, "log"),
            LogReader::new(std::fs::File::open(
                self.path.join(format!("{}.{}", new_gen, "log")),
            )?)?,
        );
        self.session_log_name = format!("{}.{}", new_gen, "log");

        let mut pos = 0;

        for (log_name, old_pos, offset) in self.store.values_mut() {
            let r = self.readers.get_mut(log_name).expect(log_name);

            r.seek(SeekFrom::Start(*old_pos as u64));
            let mut record_reader = r.take(*offset as u64);
            let len = std::io::copy(&mut record_reader, &mut comp_writer)? as usize;
            *log_name = format!("{}.{}", comp_gen, "log");
            *old_pos = pos;
            *offset = len;
            pos += len;
        }
        comp_writer.flush()?;

        let old_readers: Vec<_> = self
            .readers
            .keys()
            .filter(|gen| gen.trim_end_matches(".log").parse::<usize>().unwrap() < comp_gen)
            .cloned()
            .collect();

        for stale_gen in old_readers {
            self.readers.remove(&stale_gen);
            std::fs::remove_file(&self.path.join(stale_gen))?;
        }

        self.uncompacted = 0;

        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::fs;
//     use std::panic;
//     use tempfile::TempDir;
//     use walkdir::WalkDir;

//     fn with_tmp_dir<T>(f: T)
//     where
//         T: FnOnce(&Path) -> Result<()> + panic::UnwindSafe,
//     {
//         let tmp_dir = TempDir::new().expect("temp dir wasn't created");

//         let result = panic::catch_unwind(|| f(&tmp_dir.path()));

//         assert!(result.is_ok());
//     }

//     #[test]
//     fn test_logs_creation() {
//         with_tmp_dir(|tmp_dir: &Path| {
//             let store1 = KvStore::open(tmp_dir).expect("store wasn't created");

//             let entries = std::fs::read_dir(&tmp_dir)?.count();
//             assert_eq!(entries, 1);

//             let store2 = KvStore::open(tmp_dir)?;
//             let entries = std::fs::read_dir(&tmp_dir)?.count();
//             assert_eq!(entries, 2);

//             Ok(())
//         });
//     }
// }
