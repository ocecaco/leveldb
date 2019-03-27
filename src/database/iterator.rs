//! leveldb iterators
//!
//! Iteration is one of the most important parts of leveldb. This module provides
//! Iterators to iterate over key, values and pairs of both.
use super::options::{c_readoptions, ReadOptions};
use super::Database;
use leveldb_sys::{
    leveldb_create_iterator, leveldb_iter_destroy, leveldb_iter_key, leveldb_iter_next,
    leveldb_iter_prev, leveldb_iter_seek, leveldb_iter_seek_to_first, leveldb_iter_seek_to_last,
    leveldb_iter_valid, leveldb_iter_value, leveldb_iterator_t, leveldb_readoptions_destroy,
};
use libc::{c_char, size_t};
use std::marker::PhantomData;
use std::slice::from_raw_parts;

/// An iterator over the leveldb keyspace.
///
/// Returns key and value as a tuple.
pub struct DatabaseIterator<'a> {
    // Iterator accesses the Database through a leveldb_iter_t pointer
    // but needs to hold the reference for lifetime tracking
    #[allow(dead_code)]
    database: PhantomData<&'a Database>,
    iter: *mut leveldb_iterator_t,
}

impl<'a> DatabaseIterator<'a> {
    pub fn new(database: &'a Database, options: &ReadOptions) -> DatabaseIterator<'a> {
        unsafe {
            let c_readoptions = c_readoptions(options);
            let ptr = leveldb_create_iterator(database.database.ptr, c_readoptions);
            leveldb_readoptions_destroy(c_readoptions);
            DatabaseIterator {
                iter: ptr,
                database: PhantomData,
            }
        }
    }

    fn check_valid(&self) {
        if !self.valid() {
            panic!("disallowed method called on invalid iterator");
        }
    }

    pub fn valid(&self) -> bool {
        unsafe { leveldb_iter_valid(self.iter) != 0 }
    }

    pub fn seek_to_first(&mut self) {
        unsafe { leveldb_iter_seek_to_first(self.iter) }
    }

    pub fn seek_to_last(&mut self) {
        unsafe {
            leveldb_iter_seek_to_last(self.iter);
        }
    }

    pub fn seek(&mut self, key: &[u8]) {
        unsafe {
            leveldb_iter_seek(
                self.iter,
                key.as_ptr() as *const c_char,
                key.len() as size_t,
            );
        }
    }

    pub fn next(&mut self) {
        self.check_valid();

        unsafe {
            leveldb_iter_next(self.iter);
        }
    }

    pub fn prev(&mut self) {
        self.check_valid();

        unsafe {
            leveldb_iter_prev(self.iter);
        }
    }

    pub fn key(&self) -> &[u8] {
        self.check_valid();

        unsafe {
            let mut length: size_t = 0;
            let value = leveldb_iter_key(self.iter, &mut length) as *const u8;
            from_raw_parts(value, length as usize)
        }
    }

    pub fn value(&self) -> &[u8] {
        self.check_valid();

        unsafe {
            let mut length: size_t = 0;
            let value = leveldb_iter_value(self.iter, &mut length) as *const u8;
            from_raw_parts(value, length as usize)
        }
    }
}

#[allow(missing_docs)]
impl<'a> Drop for DatabaseIterator<'a> {
    fn drop(&mut self) {
        unsafe { leveldb_iter_destroy(self.iter) }
    }
}
