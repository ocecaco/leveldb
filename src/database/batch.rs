//! Module providing write batches

use leveldb_sys::*;
use libc::{c_char, c_void, size_t};
use std::slice;

#[allow(missing_docs)]
pub struct RawWritebatch {
    pub ptr: *mut leveldb_writebatch_t,
}

impl Drop for RawWritebatch {
    fn drop(&mut self) {
        unsafe {
            leveldb_writebatch_destroy(self.ptr);
        }
    }
}

#[allow(missing_docs)]
pub struct Writebatch {
    pub writebatch: RawWritebatch,
}

impl Writebatch {
    /// Create a new writebatch
    pub fn new() -> Writebatch {
        let ptr = unsafe { leveldb_writebatch_create() };
        let raw = RawWritebatch { ptr: ptr };
        Writebatch { writebatch: raw }
    }

    /// Clear the writebatch
    pub fn clear(&mut self) {
        unsafe { leveldb_writebatch_clear(self.writebatch.ptr) };
    }

    /// Batch a put operation
    pub fn put(&mut self, key: &[u8], value: &[u8]) {
        unsafe {
            leveldb_writebatch_put(
                self.writebatch.ptr,
                key.as_ptr() as *mut c_char,
                key.len() as size_t,
                value.as_ptr() as *mut c_char,
                value.len() as size_t,
            );
        }
    }

    /// Batch a delete operation
    pub fn delete(&mut self, key: &[u8]) {
        unsafe {
            leveldb_writebatch_delete(
                self.writebatch.ptr,
                key.as_ptr() as *mut c_char,
                key.len() as size_t,
            );
        }
    }

    /// Iterate over the writebatch, returning the resulting iterator
    pub fn iterate<T: WritebatchIterator>(&mut self, iterator: Box<T>) -> Box<T> {
        unsafe {
            let iter = Box::into_raw(iterator);
            leveldb_writebatch_iterate(
                self.writebatch.ptr,
                iter as *mut c_void,
                put_callback::<T>,
                deleted_callback::<T>,
            );
            Box::from_raw(iter)
        }
    }
}

/// A trait for iterators to iterate over written batches and check their validity.
pub trait WritebatchIterator {
    /// Callback for put items
    fn put(&mut self, key: &[u8], value: &[u8]);

    /// Callback for deleted items
    fn deleted(&mut self, key: &[u8]);
}

extern "C" fn put_callback<T: WritebatchIterator>(
    state: *mut c_void,
    key: *const i8,
    keylen: size_t,
    val: *const i8,
    vallen: size_t,
) {
    unsafe {
        let iter: &mut T = &mut *(state as *mut T);
        let key_slice = slice::from_raw_parts::<u8>(key as *const u8, keylen as usize);
        let val_slice = slice::from_raw_parts::<u8>(val as *const u8, vallen as usize);
        iter.put(key_slice, val_slice);
    }
}

extern "C" fn deleted_callback<T: WritebatchIterator>(
    state: *mut c_void,
    key: *const i8,
    keylen: size_t,
) {
    unsafe {
        let iter: &mut T = &mut *(state as *mut T);
        let key_slice = slice::from_raw_parts::<u8>(key as *const u8, keylen as usize);
        iter.deleted(key_slice);
    }
}
