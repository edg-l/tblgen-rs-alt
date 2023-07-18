use std::ffi::CStr;
use std::ffi::CString;

use crate::error::Result;
use tablegen_sys::tableGenRecordMapGet;
use tablegen_sys::tableGenRecordMapGetKeys;
use tablegen_sys::tableGenStringArrayFree;
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

        unsafe {
            // TODO: may be the cause of a double free
            // tableGenStringArrayFree(cstrs);
        }

        strings
    }
}
