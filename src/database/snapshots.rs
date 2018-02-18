//! leveldb snapshots
//!
//! Snapshots give you a reference to the database at a certain
//! point in time and won't change while you work with them.
use leveldb_sys::{leveldb_snapshot_t, leveldb_t};
use leveldb_sys::{leveldb_create_snapshot, leveldb_release_snapshot};

use database::Database;

use database::error::Error;
use database::options::ReadOptions;
use database::iterator::DatabaseIterator;

#[allow(missing_docs)]
struct RawSnapshot {
    db_ptr: *mut leveldb_t,
    ptr: *mut leveldb_snapshot_t,
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
    raw: RawSnapshot,
    database: &'a Database,
}

/// Structs implementing the Snapshots trait can be
/// snapshotted.
pub trait Snapshots {
    /// Creates a snapshot and returns a struct
    /// representing it.
    fn snapshot<'a>(&'a self) -> Snapshot<'a>;
}

impl Snapshots for Database {
    fn snapshot<'a>(&'a self) -> Snapshot<'a> {
        let db_ptr = self.database.ptr;
        let snap = unsafe { leveldb_create_snapshot(db_ptr) };

        let raw = RawSnapshot {
            db_ptr: db_ptr,
            ptr: snap,
        };
        Snapshot {
            raw: raw,
            database: self,
        }
    }
}

impl<'a> Snapshot<'a> {
    /// fetches a key from the database
    ///
    /// Inserts this snapshot into ReadOptions before reading
    pub fn get(
        &'a self,
        mut options: ReadOptions<'a>,
        key: &[u8],
    ) -> Result<Option<Vec<u8>>, Error> {
        options.snapshot = Some(self);
        self.database.get(options, key)
    }

    #[inline]
    #[allow(missing_docs)]
    pub fn raw_ptr(&self) -> *mut leveldb_snapshot_t {
        self.raw.ptr
    }

    pub fn iter(&'a self, mut options: ReadOptions<'a>) -> DatabaseIterator {
        options.snapshot = Some(self);
        self.database.iter(options)
    }
}
