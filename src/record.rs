use std::ffi::{c_uint, CStr, CString};

use tablegen_sys::{
    tableGenRecordAsNewString, tableGenRecordGetFieldType, tableGenRecordGetName,
    tableGenRecordGetValue, tableGenRecordGetValuesItr, tableGenRecordIsAnonymous,
    tableGenRecordValItrFree, tableGenRecordValItrNext, TableGenRecordRef, TableGenRecordValItrRef,
    TableGenRecordValRef,
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

    pub fn value(&self, name: &str) -> RecordValue {
        let name = CString::new(name).unwrap();
        unsafe { RecordValue::from_raw(tableGenRecordGetValue(self.raw, name.as_ptr())) }
    }

    pub fn get_field_type(&self, name: &str) -> RecordValueType {
        let name = CString::new(name).unwrap();
        unsafe { RecordValueType::from_raw(tableGenRecordGetFieldType(self.raw, name.as_ptr())) }
    }

    pub fn anonymous(&self) -> bool {
        unsafe { tableGenRecordIsAnonymous(self.raw) > 0 }
    }

    pub fn values_iter(&self) -> RecordValueIterator {
        unsafe { RecordValueIterator::from_raw(tableGenRecordGetValuesItr(self.raw)) }
    }
}

#[derive(Debug)]
pub struct RecordValue {
    raw: TableGenRecordValRef,
}

impl RecordValue {
    pub unsafe fn from_raw(ptr: TableGenRecordValRef) -> Self {
        Self { raw: ptr }
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
    raw: TableGenRecordValItrRef,
}

impl RecordValueIterator {
    unsafe fn from_raw(ptr: TableGenRecordValItrRef) -> RecordValueIterator {
        RecordValueIterator { raw: ptr }
    }
}

impl Iterator for RecordValueIterator {
    type Item = RecordValue;

    fn next(&mut self) -> Option<RecordValue> {
        let next = unsafe { tableGenRecordValItrNext(self.raw) };
        if next.is_null() {
            None
        } else {
            unsafe { Some(RecordValue::from_raw(next)) }
        }
    }
}

impl Drop for RecordValueIterator {
    fn drop(&mut self) {
        unsafe { tableGenRecordValItrFree(self.raw) }
    }
}
