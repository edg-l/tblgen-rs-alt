// Original work Copyright 2016 Alexander Stocko <as@coder.gg>.
// Modified work Copyright 2023 Daan Vanoverloop
// See the COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::{
    ffi::{c_uint, CStr, CString},
    ops::Deref,
};

use tablegen_sys::{
    tableGenRecordAsNewString, tableGenRecordGetFieldType, tableGenRecordGetFirstValue,
    tableGenRecordGetName, tableGenRecordGetValue, tableGenRecordIsAnonymous,
    tableGenRecordValGetName, tableGenRecordValGetValue, tableGenRecordValNext, TableGenRecordRef,
    TableGenRecordValRef,
};

use crate::{error, value::TypedValue};

#[derive(Debug)]
pub struct Record {
    raw: TableGenRecordRef,
}

impl Record {
    pub unsafe fn from_raw(ptr: TableGenRecordRef) -> Record {
        Record { raw: ptr }
    }

    pub fn name(&self) -> String {
        unsafe {
            CStr::from_ptr(tableGenRecordGetName(self.raw))
                .to_string_lossy()
                .into_owned()
        }
    }

    pub fn as_string(&self) -> String {
        unsafe {
            CStr::from_ptr(tableGenRecordAsNewString(self.raw))
                .to_string_lossy()
                .into_owned()
        }
    }

    pub fn value(&self, name: &str) -> error::Result<RecordValue> {
        let name = CString::new(name)?;
        unsafe { RecordValue::from_raw(tableGenRecordGetValue(self.raw, name.as_ptr())) }
    }

    pub fn field_type(&self, name: &str) -> RecordValueType {
        let name = CString::new(name).unwrap();
        unsafe { RecordValueType::from_raw(tableGenRecordGetFieldType(self.raw, name.as_ptr())) }
    }

    pub fn anonymous(&self) -> bool {
        unsafe { tableGenRecordIsAnonymous(self.raw) > 0 }
    }

    pub fn values_iter(&self) -> RecordValueIterator {
        RecordValueIterator::new(self)
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct RecordValue {
    raw: TableGenRecordValRef,
    name: String,
    value: TypedValue,
}

impl RecordValue {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn value(&self) -> &TypedValue {
        &self.value
    }

    pub unsafe fn from_raw(ptr: TableGenRecordValRef) -> error::Result<Self> {
        let name = CStr::from_ptr(tableGenRecordValGetName(ptr))
            .to_string_lossy()
            .into_owned();
        let value = TypedValue::from_typed_init(tableGenRecordValGetValue(ptr))?;
        Ok(Self {
            raw: ptr,
            name,
            value,
        })
    }
}

impl Deref for RecordValue {
    type Target = TypedValue;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

#[derive(Debug, PartialEq, PartialOrd)]
#[repr(C)]
pub enum RecordValueType {
    Bit,
    Bits,
    Code,
    Int,
    String,
    List,
    Dag,
    Record,
    Invalid,
}

impl RecordValueType {
    unsafe fn from_raw(raw: c_uint) -> Self {
        std::mem::transmute(raw)
    }
}

pub struct RecordValueIterator {
    record: TableGenRecordRef,
    current: TableGenRecordValRef,
}

impl RecordValueIterator {
    fn new(record: &Record) -> RecordValueIterator {
        unsafe {
            RecordValueIterator {
                record: record.raw,
                current: tableGenRecordGetFirstValue(record.raw),
            }
        }
    }
}

impl Iterator for RecordValueIterator {
    type Item = RecordValue;

    fn next(&mut self) -> Option<RecordValue> {
        let next = unsafe { tableGenRecordValNext(self.record, self.current) };
        self.current = next;
        if next.is_null() {
            None
        } else {
            unsafe { Some(RecordValue::from_raw(next).expect("record values are valid")) }
        }
    }
}
