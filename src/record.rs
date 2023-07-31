// Original work Copyright 2016 Alexander Stocko <as@coder.gg>.
// Modified work Copyright 2023 Daan Vanoverloop
// See the COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use paste::paste;
use std::ffi::{CStr, CString};
use std::marker::PhantomData;

use crate::raw::{
    tableGenRecordGetFirstValue, tableGenRecordGetName, tableGenRecordGetValue,
    tableGenRecordIsAnonymous, tableGenRecordIsSubclassOf, tableGenRecordValGetName,
    tableGenRecordValGetValue, tableGenRecordValNext, TableGenRecordRef, TableGenRecordValRef,
};
use crate::RecordKeeper;

use crate::error::{self, TableGenError};
use crate::init::{DagInit, ListInit, TypedInit};

#[derive(Debug, Clone, Copy)]
pub struct Record<'a> {
    raw: TableGenRecordRef,
    _reference: PhantomData<&'a RecordKeeper>,
}

macro_rules! record_value {
    ($name:ident, $type:ty) => {
        paste! {
            pub fn [<$name _value>](&self, name: &str) -> Option<$type> {
                self.value(name).ok()?.try_into().ok()
            }
        }
    };
}

impl<'a> Record<'a> {
    pub unsafe fn from_raw(ptr: TableGenRecordRef) -> Record<'a> {
        Record {
            raw: ptr,
            _reference: PhantomData,
        }
    }

    pub fn name(&self) -> String {
        unsafe {
            CStr::from_ptr(tableGenRecordGetName(self.raw))
                .to_string_lossy()
                .into_owned()
        }
    }

    record_value!(bit, i8);
    record_value!(bits, Vec<i8>);
    record_value!(code, String);
    record_value!(int, i64);
    record_value!(string, String);
    record_value!(list, ListInit);
    record_value!(dag, DagInit);
    record_value!(def, Record);

    pub fn value(&self, name: &str) -> error::Result<RecordValue> {
        let name = CString::new(name)?;
        unsafe { RecordValue::from_raw(tableGenRecordGetValue(self.raw, name.as_ptr())) }
    }

    pub fn anonymous(&self) -> bool {
        unsafe { tableGenRecordIsAnonymous(self.raw) > 0 }
    }

    pub fn subclass_of(&self, class: &str) -> bool {
        let name = CString::new(class).unwrap();
        unsafe { tableGenRecordIsSubclassOf(self.raw, name.as_ptr()) > 0 }
    }

    pub fn values(self) -> RecordValueIter<'a> {
        RecordValueIter::new(self)
    }
}

macro_rules! record_value_as {
    ($name: ident, $type:ty) => {
        paste! {
            pub fn [<as_ $name>](&self) -> Option<&$type> {
                self.value().[<as_ $name>]()
            }
        }
    };
}

macro_rules! try_into {
    ($type:ty) => {
        impl<'a> TryFrom<RecordValue<'a>> for $type {
            type Error = TableGenError;

            fn try_from(record_value: RecordValue<'a>) -> Result<Self, Self::Error> {
                record_value.value.try_into()
            }
        }
    };
}

try_into!(i8);
try_into!(Vec<i8>);
try_into!(i64);
try_into!(ListInit<'a>);
try_into!(DagInit<'a>);
try_into!(Record<'a>);
try_into!(String);

impl<'a> From<RecordValue<'a>> for TypedInit<'a> {
    fn from(value: RecordValue<'a>) -> Self {
        value.value
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RecordValue<'a> {
    raw: TableGenRecordValRef,
    name: String,
    value: TypedInit<'a>,
}

impl<'a> RecordValue<'a> {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn value(&self) -> &TypedInit {
        &self.value
    }

    record_value_as!(bit, i8);
    record_value_as!(bits, Vec<i8>);
    record_value_as!(code, String);
    record_value_as!(int, i64);
    record_value_as!(string, String);
    record_value_as!(list, ListInit);
    record_value_as!(dag, DagInit);
    record_value_as!(def, Record);

    pub unsafe fn from_raw(ptr: TableGenRecordValRef) -> error::Result<Self> {
        let name = CStr::from_ptr(tableGenRecordValGetName(ptr))
            .to_string_lossy()
            .into_owned();
        let value = TypedInit::from_raw(tableGenRecordValGetValue(ptr))?;
        Ok(Self {
            raw: ptr,
            name,
            value,
        })
    }
}

pub struct RecordValueIter<'a> {
    record: TableGenRecordRef,
    current: TableGenRecordValRef,
    _reference: PhantomData<Record<'a>>,
}

impl<'a> RecordValueIter<'a> {
    fn new(record: Record) -> RecordValueIter<'_> {
        unsafe {
            RecordValueIter {
                record: record.raw,
                current: tableGenRecordGetFirstValue(record.raw),
                _reference: PhantomData,
            }
        }
    }
}

impl<'a> Iterator for RecordValueIter<'a> {
    type Item = RecordValue<'a>;

    fn next(&mut self) -> Option<RecordValue<'a>> {
        let next = unsafe { tableGenRecordValNext(self.record, self.current) };
        self.current = next;
        if next.is_null() {
            None
        } else {
            unsafe { Some(RecordValue::from_raw(next).expect("record values are valid")) }
        }
    }
}
