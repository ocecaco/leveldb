//! A database access library for leveldb
//!
//! Usage:
//!
//! ```rust,ignore
//! extern crate tempdir;
//! extern crate leveldb;
//!
//! use tempdir::TempDir;
//! use leveldb::database::Database;
//! use leveldb::kv::KV;
//! use leveldb::options::{Options,WriteOptions,ReadOptions};
//!
//! let tempdir = TempDir::new("demo").unwrap();
//! let path = tempdir.path();
//!
//! let mut options = Options::new();
//! options.create_if_missing = true;
//! let mut database = match Database::open(path, options) {
//!     Ok(db) => { db },
//!     Err(e) => { panic!("failed to open database: {:?}", e) }
//! };
//!
//! let write_opts = WriteOptions::new();
//! match database.put(write_opts, 1, &[1]) {
//!     Ok(_) => { () },
//!     Err(e) => { panic!("failed to write to database: {:?}", e) }
//! };
//!
//! let read_opts = ReadOptions::new();
//! let res = database.get(read_opts, 1);
//!
//! match res {
//!   Ok(data) => {
//!     assert!(data.is_some());
//!     assert_eq!(data, Some(vec![1]));
//!   }
//!   Err(e) => { panic!("failed reading data: {:?}", e) }
//! }
//! ```

#![crate_type = "lib"]
#![crate_name = "leveldb"]
#![deny(missing_docs)]

use leveldb_sys::{leveldb_major_version, leveldb_minor_version};
pub use crate::database::options;
pub use crate::database::error;
pub use crate::database::iterator;
pub use crate::database::snapshots;
pub use crate::database::comparator;
pub use crate::database::batch;
pub use crate::database::management;

#[allow(missing_docs)]
pub mod database;

/// Struct containing version information of LevelDB
#[derive(Debug, Copy, Clone)]
pub struct Version {
    /// Major version
    pub major: isize,
    /// Minor version
    pub minor: isize,
}

/// Library version information
///
/// Need a recent version of leveldb to be used.
pub fn version() -> Version {
    unsafe {
        let major = leveldb_major_version() as isize;
        let minor = leveldb_minor_version() as isize;
        Version { major, minor }
    }
}
