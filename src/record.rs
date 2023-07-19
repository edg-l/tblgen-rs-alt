use std::{
    ffi::{c_uint, CStr, CString},
    ops::Deref,
};

use tablegen_sys::{
    tableGenBitArrayFree, tableGenRecordAsNewString, tableGenRecordGetFieldType,
    tableGenRecordGetFirstValue, tableGenRecordGetName, tableGenRecordGetValue,
    tableGenRecordIsAnonymous, tableGenRecordValGetName, tableGenRecordValGetType,
    tableGenRecordValGetValAsBit, tableGenRecordValGetValAsBits, tableGenRecordValGetValAsInt,
    tableGenRecordValGetValAsNewString, tableGenRecordValGetValue, tableGenRecordValNext,
    TableGenRecordRef, TableGenRecordValRef,
};

use crate::{
    error::{self, TableGenError},
    value::{DagValue, ListValue, TypedValue},
};

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
