// Original work Copyright 2016 Alexander Stocko <as@coder.gg>.
// Modified work Copyright 2023 Daan Vanoverloop
// See the COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::ffi::CStr;
use std::ffi::CString;

use tablegen_sys::tableGenRecordMapGet;
use tablegen_sys::tableGenRecordMapGetKeys;
use tablegen_sys::TableGenRecordMapRef;

use crate::record::Record;

pub struct RecordMap {
    raw: TableGenRecordMapRef,
}

impl RecordMap {
    pub fn from_raw(ptr: TableGenRecordMapRef) -> RecordMap {
        RecordMap { raw: ptr }
    }

    pub fn get(&self, name: &str) -> Option<Record> {
        unsafe {
            let name = CString::new(name).ok()?;
            let class = tableGenRecordMapGet(self.raw, name.as_ptr());
            if class.is_null() {
                None
            } else {
                Some(Record::from_raw(class))
            }
        }
    }

    pub fn keys(&self) -> Vec<String> {
        let mut len: usize = 0;
        let mut cstrs = unsafe { tableGenRecordMapGetKeys(self.raw, &mut len) };

        let mut strings: Vec<String> = Vec::new();
        for _ in 0..len {
            let s = unsafe {
                let cs = CStr::from_ptr(*cstrs).to_string_lossy().into_owned();
                cstrs = cstrs.offset(1);
                cs
            };
            strings.push(s);
        }

        strings
    }
}
