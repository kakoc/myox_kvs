// #![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
// #![cfg_attr(debug_assertions, allow(dead_code, unused_imports, warnings))]
//! core key-value storage module

#![deny(missing_docs)]

pub use kvs::KvStore;

mod kvs;
