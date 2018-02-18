//! leveldb iterators
//!
//! Iteration is one of the most important parts of leveldb. This module provides
//! Iterators to iterate over key, values and pairs of both.
use leveldb_sys::{leveldb_create_iterator, leveldb_iter_destroy, leveldb_iter_key,
                  leveldb_iter_next, leveldb_iter_seek, leveldb_iter_seek_to_first,
                  leveldb_iter_seek_to_last, leveldb_iter_valid, leveldb_iter_value,
                  leveldb_iterator_t, leveldb_readoptions_destroy};
use libc::{c_char, size_t};
use super::Database;
use super::options::{c_readoptions, ReadOptions};
use std::slice::from_raw_parts;
use std::marker::PhantomData;

#[allow(missing_docs)]
struct RawIterator {
    ptr: *mut leveldb_iterator_t,
}

#[allow(missing_docs)]
impl Drop for RawIterator {
    fn drop(&mut self) {
        unsafe { leveldb_iter_destroy(self.ptr) }
    }
}

/// An iterator over the leveldb keyspace.
///
/// Returns key and value as a tuple.
pub struct DatabaseIterator<'a> {
    start: bool,
    // Iterator accesses the Database through a leveldb_iter_t pointer
    // but needs to hold the reference for lifetime tracking
    #[allow(dead_code)] database: PhantomData<&'a Database>,
    iter: RawIterator,
    from: Option<&'a [u8]>,
    to: Option<&'a [u8]>,
}

impl<'a> DatabaseIterator<'a> {
    pub fn new(database: &'a Database, options: ReadOptions<'a>) -> DatabaseIterator<'a> {
        unsafe {
            let c_readoptions = c_readoptions(&options);
            let ptr = leveldb_create_iterator(database.database.ptr, c_readoptions);
            leveldb_readoptions_destroy(c_readoptions);
            leveldb_iter_seek_to_first(ptr);
            DatabaseIterator {
                start: true,
                iter: RawIterator { ptr: ptr },
                database: PhantomData,
                from: None,
                to: None,
            }
        }
    }

    /// return the last element of the iterator
    pub fn last(self) -> Option<(Vec<u8>, Vec<u8>)> {
        self.seek_to_last();
        Some((self.key(), self.value()))
    }

    fn valid(&self) -> bool {
        unsafe { leveldb_iter_valid(self.raw_iterator()) != 0 }
    }

    fn advance(&mut self) -> bool {
        unsafe {
            if !self.start() {
                leveldb_iter_next(self.raw_iterator());
            } else {
                if let Some(k) = self.from_key() {
                    self.seek(k)
                }
                self.started();
            }
        }
        self.valid()
    }

    fn key(&self) -> Vec<u8> {
        unsafe {
            let length: size_t = 0;
            let value = leveldb_iter_key(self.raw_iterator(), &length) as *const u8;
            from_raw_parts(value, length as usize).into()
        }
    }

    fn value(&self) -> Vec<u8> {
        unsafe {
            let length: size_t = 0;
            let value = leveldb_iter_value(self.raw_iterator(), &length) as *const u8;
            from_raw_parts(value, length as usize).to_vec()
        }
    }

    pub fn seek_to_first(&self) {
        unsafe { leveldb_iter_seek_to_first(self.raw_iterator()) }
    }

    pub fn seek_to_last(&self) {
        if let Some(k) = self.to_key() {
            self.seek(k);
        } else {
            unsafe {
                leveldb_iter_seek_to_last(self.raw_iterator());
            }
        }
    }

    pub fn seek(&self, key: &[u8]) {
        unsafe {
            leveldb_iter_seek(
                self.raw_iterator(),
                key.as_ptr() as *mut c_char,
                key.len() as size_t,
            );
        }
    }

    #[inline]
    fn raw_iterator(&self) -> *mut leveldb_iterator_t {
        self.iter.ptr
    }

    #[inline]
    fn start(&self) -> bool {
        self.start
    }

    #[inline]
    fn started(&mut self) {
        self.start = false
    }

    pub fn from(mut self, key: &'a [u8]) -> Self {
        self.from = Some(key);
        self
    }

    pub fn to(mut self, key: &'a [u8]) -> Self {
        self.to = Some(key);
        self
    }

    fn from_key(&self) -> Option<&[u8]> {
        self.from
    }

    fn to_key(&self) -> Option<&[u8]> {
        self.to
    }
}

impl<'a> Iterator for DatabaseIterator<'a> {
    type Item = (Vec<u8>, Vec<u8>);

    fn next(&mut self) -> Option<(Vec<u8>, Vec<u8>)> {
        if self.advance() {
            Some((self.key(), self.value()))
        } else {
            None
        }
    }
}
