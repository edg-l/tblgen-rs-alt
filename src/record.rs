use std::{
    ffi::{c_uint, CStr, CString},
    ops::Deref,
};

use tablegen_sys::{
    tableGenBitArrayFree, tableGenRecordAsNewString, tableGenRecordGetFieldType,
    tableGenRecordGetName, tableGenRecordGetValue, tableGenRecordGetValuesItr,
    tableGenRecordIsAnonymous, tableGenRecordValGetName, tableGenRecordValGetType,
    tableGenRecordValGetValAsBit, tableGenRecordValGetValAsBits, tableGenRecordValGetValAsInt,
    tableGenRecordValGetValAsNewString, tableGenRecordValGetValAsRecord, tableGenRecordValItrFree,
    tableGenRecordValItrNext, TableGenRecordRef, TableGenRecordValItrRef, TableGenRecordValRef,
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
        unsafe { RecordValueIterator::from_raw(tableGenRecordGetValuesItr(self.raw)) }
    }
}

#[derive(Debug)]
pub struct RecordValue {
    raw: TableGenRecordValRef,
    name: String,
    value: TypedValue,
}

impl RecordValue {
    pub unsafe fn from_raw(ptr: TableGenRecordValRef) -> error::Result<Self> {
        let value_type = tableGenRecordValGetType(ptr);
        let name = CStr::from_ptr(tableGenRecordValGetName(ptr))
            .to_string_lossy()
            .into_owned();
        use tablegen_sys::TableGenRecTyKind::*;
        let value = match value_type {
            TableGenBitRecTyKind => {
                let mut bit = -1;
                tableGenRecordValGetValAsBit(ptr, &mut bit);

                if bit == 0 || bit == 1 {
                    Ok(TypedValue::Bit(bit))
                } else {
                    Err(TableGenError::InvalidBitRange)
                }
            }
            TableGenBitsRecTyKind => {
                let mut bits: Vec<_> = Vec::new();
                let mut len: usize = 0;
                let cbits = tableGenRecordValGetValAsBits(ptr, &mut len);
                let mut bits_ptr = cbits;
                for _ in 0..len {
                    bits.push(*bits_ptr);
                    bits_ptr = bits_ptr.offset(1);
                }
                tableGenBitArrayFree(cbits);
                if bits.is_empty() {
                    Err(TableGenError::NullPointer.into())
                } else {
                    Ok(TypedValue::Bits(bits))
                }
            }
            TableGenDagRecTyKind => Ok(TypedValue::Dag(DagValue::from_ptr(ptr))),
            TableGenIntRecTyKind => {
                let mut int: i64 = 0;
                tableGenRecordValGetValAsInt(ptr, &mut int);
                Ok(TypedValue::Int(int))
            }
            TableGenListRecTyKind => Ok(TypedValue::List(ListValue::from_ptr(ptr))),
            // TableGenRecordRecTyKind => Ok(TypedValue::Record(Record::from_raw(
            //     tableGenRecordValGetValAsRecord(ptr),
            // ))),
            TableGenStringRecTyKind => {
                let cstr = tableGenRecordValGetValAsNewString(ptr);
                Ok(TypedValue::String(
                    CStr::from_ptr(cstr).to_string_lossy().into_owned(),
                ))
            }
            _ => Ok(TypedValue::Invalid),
        }?;
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
            unsafe { Some(RecordValue::from_raw(next).expect("record values are valid")) }
        }
    }
}

impl Drop for RecordValueIterator {
    fn drop(&mut self) {
        unsafe { tableGenRecordValItrFree(self.raw) }
    }
}
