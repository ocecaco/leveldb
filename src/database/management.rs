//! Management functions, e.g. for destroying and reparing a database.
use crate::error::Error;
use crate::options::{c_options, Options};
use std::ffi::CString;
use std::path::Path;
use std::ptr;

use leveldb_sys::{leveldb_destroy_db, leveldb_repair_db};

/// destroy a database. You shouldn't hold a handle on the database anywhere at that time.
pub fn destroy(name: &Path, options: &Options) -> Result<(), Error> {
    let mut error = ptr::null_mut();
    unsafe {
        let c_string = CString::new(name.to_str().unwrap()).unwrap();
        let c_options = c_options(options, None);
        leveldb_destroy_db(
            c_options,
            c_string.as_bytes_with_nul().as_ptr() as *const i8,
            &mut error,
        );

        if error.is_null() {
            Ok(())
        } else {
            Err(Error::new_from_i8(error))
        }
    }
}

/// repair the database. The database should be closed at this moment.
pub fn repair(name: &Path, options: &Options) -> Result<(), Error> {
    let mut error = ptr::null_mut();
    unsafe {
        let c_string = CString::new(name.to_str().unwrap()).unwrap();
        let c_options = c_options(options, None);
        leveldb_repair_db(
            c_options,
            c_string.as_bytes_with_nul().as_ptr() as *const i8,
            &mut error,
        );

        if error.is_null() {
            Ok(())
        } else {
            Err(Error::new_from_i8(error))
        }
    }
}
