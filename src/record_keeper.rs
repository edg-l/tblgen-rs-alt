// Original work Copyright 2016 Alexander Stocko <as@coder.gg>.
// Modified work Copyright 2023 Daan Vanoverloop
// See the COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::ffi::{CStr, CString};
use std::marker::PhantomData;

use crate::raw::{
    tableGenRecordKeeperGetAllDerivedDefinitions, tableGenRecordKeeperGetClass,
    tableGenRecordKeeperGetDef, tableGenRecordKeeperGetFirstClass, tableGenRecordKeeperGetFirstDef,
    tableGenRecordKeeperGetNextClass, tableGenRecordKeeperGetNextDef,
    tableGenRecordKeeperItemGetName, tableGenRecordKeeperItemGetRecord,
    tableGenRecordKeeperIteratorClone, tableGenRecordKeeperIteratorFree, tableGenRecordVectorFree,
    tableGenRecordVectorGet, TableGenRecordKeeperIteratorRef, TableGenRecordKeeperRef,
    TableGenRecordVectorRef,
};
use crate::record::Record;
use crate::TableGen;

#[derive(Clone, Copy)]
pub struct RecordKeeperRef<'a> {
    raw: TableGenRecordKeeperRef,
    _reference: PhantomData<&'a TableGen>,
}

impl<'a> RecordKeeperRef<'a> {
    pub unsafe fn from_raw(ptr: TableGenRecordKeeperRef) -> RecordKeeperRef<'a> {
        RecordKeeperRef {
            raw: ptr,
            _reference: PhantomData,
        }
    }

    pub fn classes(&self) -> NamedRecordIter<'_, IsClass> {
        unsafe { NamedRecordIter::from_raw(tableGenRecordKeeperGetFirstClass(self.raw)) }
    }

    pub fn defs(&self) -> NamedRecordIter<'_, IsDef> {
        unsafe { NamedRecordIter::from_raw(tableGenRecordKeeperGetFirstDef(self.raw)) }
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

    pub fn all_derived_definitions(&self, name: &str) -> RecordIter {
        let name = CString::new(name).unwrap();
        unsafe {
            RecordIter::from_raw_vector(tableGenRecordKeeperGetAllDerivedDefinitions(
                self.raw,
                name.as_ptr(),
            ))
        }
    }
}

#[doc(hidden)]
pub struct IsClass;
#[doc(hidden)]
pub struct IsDef;

trait NextRecord {
    unsafe fn next(raw: &mut TableGenRecordKeeperIteratorRef);
}

impl NextRecord for IsClass {
    unsafe fn next(raw: &mut TableGenRecordKeeperIteratorRef) {
        tableGenRecordKeeperGetNextClass(raw);
    }
}

impl NextRecord for IsDef {
    unsafe fn next(raw: &mut TableGenRecordKeeperIteratorRef) {
        tableGenRecordKeeperGetNextDef(raw);
    }
}

#[derive(Debug)]
pub struct NamedRecordIter<'a, T> {
    raw: TableGenRecordKeeperIteratorRef,
    _kind: PhantomData<T>,
    _reference: PhantomData<RecordKeeperRef<'a>>,
}

impl<'a, T> NamedRecordIter<'a, T> {
    unsafe fn from_raw(raw: TableGenRecordKeeperIteratorRef) -> Self {
        NamedRecordIter {
            raw,
            _kind: PhantomData,
            _reference: PhantomData,
        }
    }
}

impl<'a, T: NextRecord> Iterator for NamedRecordIter<'a, T> {
    type Item = (String, Record);

    fn next(&mut self) -> Option<(String, Record)> {
        let current = if self.raw.is_null() {
            return None;
        } else {
            unsafe {
                Some((
                    CStr::from_ptr(tableGenRecordKeeperItemGetName(self.raw))
                        .to_string_lossy()
                        .into_owned(),
                    Record::from_raw(tableGenRecordKeeperItemGetRecord(self.raw)),
                ))
            }
        };
        unsafe { T::next(&mut self.raw) };
        current
    }
}

impl<'a, T> Clone for NamedRecordIter<'a, T> {
    fn clone(&self) -> Self {
        unsafe { Self::from_raw(tableGenRecordKeeperIteratorClone(self.raw)) }
    }
}

impl<'a, T> Drop for NamedRecordIter<'a, T> {
    fn drop(&mut self) {
        unsafe { tableGenRecordKeeperIteratorFree(self.raw) }
    }
}

pub struct RecordIter {
    raw: TableGenRecordVectorRef,
    index: usize,
}

impl RecordIter {
    unsafe fn from_raw_vector(ptr: TableGenRecordVectorRef) -> RecordIter {
        RecordIter { raw: ptr, index: 0 }
    }
}

impl Iterator for RecordIter {
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

impl Drop for RecordIter {
    fn drop(&mut self) {
        unsafe { tableGenRecordVectorFree(self.raw) }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn classes_and_defs() {
        let tablegen = TableGen::new(
            r#"
            class A;
            class B;
            class C;

            def D1: A;
            def D2: B;
            def D3: C;
        "#,
            &[],
        )
        .expect("valid tablegen");
        let rk = tablegen.record_keeper();
        rk.classes().for_each(|i| assert!(i.1.name() == i.0));
        rk.defs().for_each(|i| assert!(i.1.name() == i.0));
        assert!(rk.classes().map(|i| i.0).eq(["A", "B", "C"]));
        assert!(rk.defs().map(|i| i.0).eq(["D1", "D2", "D3"]));
    }

    #[test]
    fn derived_defs() {
        let tablegen = TableGen::new(
            r#"
            class A;
            class B;
            class C;

            def D1: A;
            def D2: A, B;
            def D3: B, C;
        "#,
            &[],
        )
        .expect("valid tablegen");
        let rk = tablegen.record_keeper();
        let a = rk.all_derived_definitions("A");
        assert!(a.map(|i| i.name()).eq(["D1", "D2"]));
        let b = rk.all_derived_definitions("B");
        assert!(b.map(|i| i.name()).eq(["D2", "D3"]));
    }

    #[test]
    fn single() {
        let tablegen = TableGen::new(
            r#"
            class A;
            def D1;
        "#,
            &[],
        )
        .expect("valid tablegen");
        let rk = tablegen.record_keeper();
        assert_eq!(rk.class("A").expect("class exists").name(), "A");
        assert_eq!(rk.def("D1").expect("def exists").name(), "D1");
    }
}
