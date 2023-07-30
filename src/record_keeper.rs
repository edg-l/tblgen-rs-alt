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
    tableGenRecordKeeperItemGetName, tableGenRecordKeeperItemGetRecord, tableGenRecordVectorFree,
    tableGenRecordVectorGet, TableGenRecordKeeperIteratorRef, TableGenRecordKeeperRef,
    TableGenRecordVectorRef,
};
use crate::record::Record;
use crate::TableGen;

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

    pub fn classes(&self) -> NamedRecordIterator<'_, IsClass> {
        unsafe { NamedRecordIterator::from_raw(tableGenRecordKeeperGetFirstClass(self.raw)) }
    }

    pub fn defs(&self) -> NamedRecordIterator<'_, IsDef> {
        unsafe { NamedRecordIterator::from_raw(tableGenRecordKeeperGetFirstDef(self.raw)) }
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

pub struct IsClass;
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

pub struct NamedRecordIterator<'a, T> {
    raw: TableGenRecordKeeperIteratorRef,
    _kind: PhantomData<T>,
    _reference: PhantomData<RecordKeeperRef<'a>>,
}

impl<'a, T> NamedRecordIterator<'a, T> {
    unsafe fn from_raw(raw: TableGenRecordKeeperIteratorRef) -> Self {
        NamedRecordIterator {
            raw,
            _kind: PhantomData,
            _reference: PhantomData,
        }
    }
}

impl<'a, T: NextRecord> Iterator for NamedRecordIterator<'a, T> {
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

#[cfg(test)]
mod test {
    use crate::TableGen;

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
