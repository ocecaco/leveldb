//! The main database module, allowing to interface with leveldb on
//! a key-value basis.
use self::bytes::Bytes;
use self::options::{c_options, Options};
use leveldb_sys::*;
use libc::{c_char, size_t};
use std::ffi::CString;

use self::error::Error;
use crate::options::{c_readoptions, c_writeoptions, ReadOptions, WriteOptions};

use std::path::Path;

use crate::iterator::DatabaseIterator;
use std::ptr;

pub mod bytes;
pub mod error;
pub mod iterator;
pub mod management;
pub mod options;

#[allow(missing_docs)]
struct RawDB {
    ptr: *mut leveldb_t,
}

#[allow(missing_docs)]
impl Drop for RawDB {
    fn drop(&mut self) {
        unsafe {
            leveldb_close(self.ptr);
        }
    }
}

/// The main database object.
///
/// leveldb databases are based on ordered keys. By default, leveldb orders
/// by the binary value of the key. Additionally, a custom `Comparator` can
/// be passed when opening the database. This library ships with an Comparator
/// implementation for keys that are `Ord`.
///
/// When re-CString a database, you must use the same key type `K` and
/// comparator type `C`.
///
/// Multiple Database objects can be kept around, as leveldb synchronises
/// internally.
pub struct Database {
    database: RawDB,
}

unsafe impl Sync for Database {}
unsafe impl Send for Database {}

impl Database {
    unsafe fn new(database: *mut leveldb_t) -> Database {
        Database {
            database: RawDB { ptr: database },
        }
    }

    /// Open a new database
    ///
    /// If the database is missing, the behaviour depends on `options.create_if_missing`.
    /// The database will be created using the settings given in `options`.
    pub fn open(name: &Path, options: Options) -> Result<Database, Error> {
        let mut error = ptr::null_mut();
        unsafe {
            let c_string = CString::new(name.to_str().unwrap()).unwrap();
            let c_options = c_options(&options, None);
            let db = leveldb_open(
                c_options as *const leveldb_options_t,
                c_string.as_bytes_with_nul().as_ptr() as *const i8,
                &mut error,
            );
            leveldb_options_destroy(c_options);

            if error.is_null() {
                Ok(Database::new(db))
            } else {
                Err(Error::new_from_i8(error))
            }
        }
    }

    /// put a binary value into the database.
    ///
    /// If the key is already present in the database, it will be overwritten.
    ///
    /// The passed key will be compared using the comparator.
    ///
    /// The database will be synced to disc if `options.sync == true`. This is
    /// NOT the default.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn put(&self, options: &WriteOptions, key: &[u8], value: &[u8]) -> Result<(), Error> {
        unsafe {
            let mut error = ptr::null_mut();
            let c_writeoptions = c_writeoptions(options);
            leveldb_put(
                self.database.ptr,
                c_writeoptions,
                key.as_ptr() as *mut c_char,
                key.len() as size_t,
                value.as_ptr() as *mut c_char,
                value.len() as size_t,
                &mut error,
            );
            leveldb_writeoptions_destroy(c_writeoptions);

            if error.is_null() {
                Ok(())
            } else {
                Err(Error::new_from_i8(error))
            }
        }
    }

    /// delete a value from the database.
    ///
    /// The passed key will be compared using the comparator.
    ///
    /// The database will be synced to disc if `options.sync == true`. This is
    /// NOT the default.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn delete(&self, options: &WriteOptions, key: &[u8]) -> Result<(), Error> {
        unsafe {
            let mut error = ptr::null_mut();
            let c_writeoptions = c_writeoptions(options);
            leveldb_delete(
                self.database.ptr,
                c_writeoptions,
                key.as_ptr() as *mut c_char,
                key.len() as size_t,
                &mut error,
            );
            leveldb_writeoptions_destroy(c_writeoptions);
            if error.is_null() {
                Ok(())
            } else {
                Err(Error::new_from_i8(error))
            }
        }
    }

    pub fn get_bytes(&self, options: &ReadOptions, key: &[u8]) -> Result<Option<Bytes>, Error> {
        unsafe {
            let mut error = ptr::null_mut();
            let mut length: size_t = 0;
            let c_readoptions = c_readoptions(options);
            let result = leveldb_get(
                self.database.ptr,
                c_readoptions,
                key.as_ptr() as *mut c_char,
                key.len() as size_t,
                &mut length,
                &mut error,
            );
            leveldb_readoptions_destroy(c_readoptions);

            if error.is_null() {
                Ok(Bytes::from_raw(result as *mut u8, length))
            } else {
                Err(Error::new_from_i8(error))
            }
        }
    }

    pub fn iter<'a>(&'a self, options: &ReadOptions) -> DatabaseIterator<'a> {
        DatabaseIterator::new(self, options)
    }

    pub fn compact(&self, start: &[u8], limit: &[u8]) {
        unsafe {
            leveldb_compact_range(
                self.database.ptr,
                start.as_ptr() as *mut c_char,
                start.len() as size_t,
                limit.as_ptr() as *mut c_char,
                limit.len() as size_t,
            );
        }
    }
}
