// Original work Copyright 2016 Alexander Stocko <as@coder.gg>.
// Modified work Copyright 2023 Daan Vanoverloop
// See the COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::ffi::CString;

use crate::raw::{
    tableGenRecordKeeperGetAllDerivedDefinitions, tableGenRecordKeeperGetClass,
    tableGenRecordKeeperGetClasses, tableGenRecordKeeperGetDef, tableGenRecordKeeperGetDefs,
    tableGenRecordVectorFree, tableGenRecordVectorGet, TableGenRecordKeeperRef,
    TableGenRecordVectorRef,
};
use crate::{record::Record, record_map::RecordMap};

pub struct RecordKeeper {
    raw: TableGenRecordKeeperRef,
}

impl RecordKeeper {
    pub unsafe fn from_raw(ptr: TableGenRecordKeeperRef) -> RecordKeeper {
        RecordKeeper { raw: ptr }
    }

    pub fn classes(&self) -> RecordMap {
        RecordMap::from_raw(unsafe { tableGenRecordKeeperGetClasses(self.raw) })
    }

    pub fn defs(&self) -> RecordMap {
        RecordMap::from_raw(unsafe { tableGenRecordKeeperGetDefs(self.raw) })
    }

    pub fn class(&self, name: &str) -> Option<Record> {
        unsafe {
            let name = CString::new(name).ok()?;
            let class = tableGenRecordKeeperGetClass(self.raw, name.as_ptr());
            if class.is_null() {
                None
            } else {
                Some(Record::from_raw(class))
            }
        }
    }

    pub fn def(&self, name: &str) -> Option<Record> {
        unsafe {
            let name = CString::new(name).ok()?;
            let def = tableGenRecordKeeperGetDef(self.raw, name.as_ptr());
            if def.is_null() {
                None
            } else {
                Some(Record::from_raw(def))
            }
        }
    }

    pub fn all_derived_definitions(&self, name: &str) -> RecordIterator {
        let name = CString::new(name).unwrap();
        unsafe {
            RecordIterator::from_raw_vector(tableGenRecordKeeperGetAllDerivedDefinitions(
                self.raw,
                name.as_ptr(),
            ))
        }
    }
}

pub struct RecordIterator {
    raw: TableGenRecordVectorRef,
    index: usize,
}

impl RecordIterator {
    unsafe fn from_raw_vector(ptr: TableGenRecordVectorRef) -> RecordIterator {
        RecordIterator { raw: ptr, index: 0 }
    }
}

impl Iterator for RecordIterator {
    type Item = Record;

    fn next(&mut self) -> Option<Record> {
        let next = unsafe { tableGenRecordVectorGet(self.raw, self.index) };
        self.index += 1;
        if next.is_null() {
            None
        } else {
            unsafe { Some(Record::from_raw(next)) }
        }
    }
}

impl Drop for RecordIterator {
    fn drop(&mut self) {
        unsafe { tableGenRecordVectorFree(self.raw) }
    }
}
