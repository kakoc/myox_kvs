// #![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
#![cfg_attr(debug_assertions, allow(dead_code, unused_imports, warnings))]
//! core key-value storage module

// #![deny(missing_docs)]

pub use crate::engine::KvsEngine;
pub use crate::kvs::{KvStore, Result};
pub use crate::sled_engine::SledKvsEngine;

mod engine;
mod error;
mod kvs;
mod log;
mod sled_engine;
