use sled;

use std::collections::HashMap;
use std::ffi::OsString;
use std::fs::{read_dir, File, ReadDir};
use std::io::{copy, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use crate::{
    error::Error,
    kvs::KvStoreValue,
    log::{create_log_file, LogCommand, LogReader, LogWriter},
    KvStore, KvsEngine, Result,
};

pub struct SledKvsEngine {
    store: sled::Db,
}

impl SledKvsEngine {
    pub fn new(path: &PathBuf) -> Self {
        SledKvsEngine {
            store: sled::open(path).unwrap(),
        }
    }
}

impl KvsEngine for SledKvsEngine {
    fn get(&mut self, key: String) -> Result<Option<String>> {
        if let Ok(Some(s)) = self.store.get(key) {
            let s = std::str::from_utf8(&s[..])
                .expect("sled stores a correct utf8 string")
                .to_string();

            Ok(Some(s))
        } else {
            Err(Error::KeyNotFound)
        }
    }

    fn set(&mut self, key: String, value: String) -> Result<()> {
        if let Ok(_) = self.store.set(key.as_bytes(), value.as_bytes()) {
            self.store.flush();
            Ok(())
        } else {
            Err(Error::InsertError)
        }
    }

    fn remove(&mut self, key: String) -> Result<()> {
        if let Ok(Some(_)) = self.store.remove(key.as_bytes()) {
            self.store.flush();
            Ok(())
        } else {
            Err(Error::RemoveError)
        }
    }
}
