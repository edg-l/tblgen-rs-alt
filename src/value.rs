// Original work Copyright 2016 Alexander Stocko <as@coder.gg>.
// Modified work Copyright 2023 Daan Vanoverloop
// See the COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use tablegen_sys::{
    tableGenBitArrayFree, tableGenBitInitGetValue, tableGenBitsInitGetValue, tableGenDagRecordGet,
    tableGenDefInitGetValue, tableGenInitRecType, tableGenIntInitGetValue, tableGenListRecordGet,
    tableGenStringInitGetValueNewString, TableGenTypedInitRef,
};

use crate::{
    error::{self, TableGenError},
    record::Record,
};
use std::ffi::CStr;

#[derive(Debug)]
pub enum TypedValue {
    Bit(i8),
    Bits(Vec<i8>),
    Code(String),
    Int(i64),
    String(String),
    List(ListValue),
    Dag(DagValue),
    Record(Record),
    Invalid,
}

impl TypedValue {
    #[allow(non_upper_case_globals)]
    pub unsafe fn from_typed_init(init: TableGenTypedInitRef) -> error::Result<Self> {
        let t = tableGenInitRecType(init);

        use tablegen_sys::TableGenRecTyKind::*;
        match t {
            TableGenBitRecTyKind => {
                let mut bit = -1;
                tableGenBitInitGetValue(init, &mut bit);

                if bit == 0 || bit == 1 {
                    Ok(TypedValue::Bit(bit))
                } else {
                    Err(TableGenError::InvalidBitRange)
                }
            }
            TableGenBitsRecTyKind => {
                let mut bits: Vec<_> = Vec::new();
                let mut len: usize = 0;
                let cbits = tableGenBitsInitGetValue(init, &mut len);
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
            TableGenDagRecTyKind => Ok(TypedValue::Dag(DagValue::from_raw(init))),
            TableGenIntRecTyKind => {
                let mut int: i64 = 0;
                tableGenIntInitGetValue(init, &mut int);
                Ok(TypedValue::Int(int))
            }
            TableGenListRecTyKind => Ok(TypedValue::List(ListValue::from_raw(init))),
            TableGenRecordRecTyKind => Ok(TypedValue::Record(Record::from_raw(
                tableGenDefInitGetValue(init),
            ))),
            TableGenStringRecTyKind => {
                let cstr = tableGenStringInitGetValueNewString(init);
                Ok(TypedValue::String(
                    CStr::from_ptr(cstr).to_string_lossy().into_owned(),
                ))
            }
            _ => Ok(Self::Invalid),
        }
    }
}

#[derive(Debug)]
pub struct DagValue {
    raw: TableGenTypedInitRef,
}

impl DagValue {
    pub fn from_raw(val: TableGenTypedInitRef) -> DagValue {
        DagValue { raw: val }
    }

    pub fn values_iter(&self) -> DagIterator {
        DagIterator::from_raw(self.raw)
    }
}

pub struct DagIterator {
    raw: TableGenTypedInitRef,
    index: usize,
}

impl DagIterator {
    fn from_raw(raw: TableGenTypedInitRef) -> DagIterator {
        DagIterator { raw, index: 0 }
    }
}

impl Iterator for DagIterator {
    type Item = TypedValue;

    fn next(&mut self) -> Option<TypedValue> {
        let next = unsafe { tableGenDagRecordGet(self.raw, self.index) };
        self.index += 1;
        if !next.is_null() {
            unsafe { TypedValue::from_typed_init(next).ok() }
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct ListValue {
    raw: TableGenTypedInitRef,
}

impl ListValue {
    pub fn from_raw(val: TableGenTypedInitRef) -> ListValue {
        ListValue { raw: val }
    }

    pub fn values_iter(&self) -> ListIterator {
        ListIterator::from_raw(self.raw)
    }
}

pub struct ListIterator {
    raw: TableGenTypedInitRef,
    index: usize,
}

impl ListIterator {
    fn from_raw(raw: TableGenTypedInitRef) -> ListIterator {
        ListIterator { raw, index: 0 }
    }
}

impl Iterator for ListIterator {
    type Item = TypedValue;

    fn next(&mut self) -> Option<TypedValue> {
        let next = unsafe { tableGenListRecordGet(self.raw, self.index) };
        self.index += 1;
        if !next.is_null() {
            unsafe { TypedValue::from_typed_init(next).ok() }
        } else {
            None
        }
    }
}
