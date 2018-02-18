//! Compaction
use super::Database;
use leveldb_sys::leveldb_compact_range;
use libc::{c_char, size_t};

pub trait Compaction<'a> {
    fn compact(&self, start: &'a [u8], limit: &'a [u8]);
}

impl<'a> Compaction<'a> for Database {
    fn compact(&self, start: &'a [u8], limit: &'a [u8]) {
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
