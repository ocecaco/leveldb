//! leveldb snapshots
//!
//! Snapshots give you a reference to the database at a certain
//! point in time and won't change while you work with them.
use leveldb_sys::{leveldb_snapshot_t, leveldb_t};
use leveldb_sys::leveldb_release_snapshot;

use crate::database::Database;

use crate::database::error::Error;
use crate::database::options::ReadOptions;
use crate::database::iterator::DatabaseIterator;
use crate::database::bytes::Bytes;

#[allow(missing_docs)]
pub struct RawSnapshot {
    pub db_ptr: *mut leveldb_t,
    pub ptr: *mut leveldb_snapshot_t,
}

impl Drop for RawSnapshot {
    fn drop(&mut self) {
        unsafe { leveldb_release_snapshot(self.db_ptr, self.ptr) };
    }
}

/// A database snapshot
///
/// Represents a database at a certain point in time,
/// and allows for all read operations (get and iteration).
pub struct Snapshot<'a> {
    pub raw: RawSnapshot,
    pub database: &'a Database,
}

impl<'a> Snapshot<'a> {
    /// fetches a key from the database
    ///
    /// Inserts this snapshot into ReadOptions before reading
    pub fn get(
        &'a self,
        options: &mut ReadOptions<'a>,
        key: &[u8],
    ) -> Result<Option<Bytes>, Error> {
        options.snapshot = Some(self);
        self.database.get_bytes(options, key)
    }

    #[inline]
    #[allow(missing_docs)]
    pub fn raw_ptr(&self) -> *mut leveldb_snapshot_t {
        self.raw.ptr
    }

    pub fn iter(&'a self, options: &mut ReadOptions<'a>) -> DatabaseIterator<'a> {
        options.snapshot = Some(self);
        self.database.iter(options)
    }
}
