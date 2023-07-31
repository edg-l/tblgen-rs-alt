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
use std::marker::PhantomData;
use std::str::Utf8Error;

use crate::raw::{
    tableGenRecordGetFirstValue, tableGenRecordGetName, tableGenRecordGetValue,
    tableGenRecordIsAnonymous, tableGenRecordIsSubclassOf, tableGenRecordValGetNameInit,
    tableGenRecordValGetValue, tableGenRecordValNext, TableGenRecordRef, TableGenRecordValRef,
};
use crate::RecordKeeper;

use crate::error::TableGenError;
use crate::init::{BitInit, BitsInit, DagInit, DefInit, IntInit, ListInit, StringInit, TypedInit};
use crate::string_ref::StringRef;

/// An immutable reference to a TableGen record.
///
/// This reference cannot outlive the [`RecordKeeper`] from which it is
/// borrowed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Record<'a> {
    raw: TableGenRecordRef,
    _reference: PhantomData<&'a RecordKeeper>,
}

macro_rules! record_value {
    ($(#[$attr:meta])* $name:ident, $type:ty) => {
        paste! {
            $(#[$attr])*
            pub fn [<$name _value>](&self, name: &str) -> Option<$type> {
                self.value(name)?.try_into().ok()
            }
        }
    };
}

impl<'a> Record<'a> {
    /// Creates a record from a raw object.
    ///
    /// # Safety
    ///
    /// The raw object must be valid.
    pub unsafe fn from_raw(ptr: TableGenRecordRef) -> Record<'a> {
        Record {
            raw: ptr,
            _reference: PhantomData,
        }
    }

    /// Returns the name of the record.
    ///
    /// # Errors
    ///
    /// Returns a [`Utf8Error`] if the name is not a valid UTF-8 string.
    pub fn name(&self) -> Result<&str, Utf8Error> {
        unsafe { StringRef::from_raw(tableGenRecordGetName(self.raw)) }.try_into()
    }

    record_value!(
        /// Returns the boolean value of the field with the given name if this
        /// field is of type [`BitInit`].
        bit,
        bool
    );
    record_value!(
        /// Returns the field with the given name converted to a [`Vec<bool>`]
        /// if this field is of type [`BitsInit`].
        bits,
        Vec<bool>
    );
    record_value!(
        /// Returns the integer value of the field with the given name if this
        /// field is of type [`IntInit`].
        int,
        i64
    );
    record_value!(
        /// Returns the field with the given name converted to a [`String`]
        /// if this field is of type [`StringInit`].
        ///
        /// Note that this copies the string into a new string.
        code,
        String
    );
    record_value!(
        /// Returns the field with the given name converted to a [`&str`]
        /// if this field is of type [`StringInit`].
        code_str,
        &str
    );
    record_value!(
        /// Returns the field with the given name converted to a [`String`]
        /// if this field is of type [`StringInit`].
        ///
        /// Note that this copies the string into a new string.
        string,
        String
    );
    record_value!(
        /// Returns the field with the given name converted to a [`&str`]
        /// if this field is of type [`StringInit`].
        str,
        &str
    );
    record_value!(
        /// Returns the field with the given name converted to a [`Record`]
        /// if this field is of type [`DefInit`].
        def,
        Record
    );
    record_value!(
        /// Returns the field with the given name converted to a [`ListInit`]
        /// if this field is of type [`ListInit`].
        list,
        ListInit
    );
    record_value!(
        /// Returns the field with the given name converted to a [`DagInit`]
        /// if this field is of type [`DagInit`].
        dag,
        DagInit
    );

    /// Returns a [`RecordValue`] for the field with the given name.
    pub fn value(&self, name: &str) -> Option<RecordValue> {
        unsafe {
            let value = tableGenRecordGetValue(self.raw, StringRef::from(name).to_raw());
            if !value.is_null() {
                Some(RecordValue::from_raw(value))
            } else {
                None
            }
        }
    }

    /// Returns true if the record is anonymous.
    pub fn anonymous(&self) -> bool {
        unsafe { tableGenRecordIsAnonymous(self.raw) > 0 }
    }

    /// Returns true if the record is a subclass of the class with the given
    /// name.
    pub fn subclass_of(&self, class: &str) -> bool {
        unsafe { tableGenRecordIsSubclassOf(self.raw, StringRef::from(class).to_raw()) > 0 }
    }

    /// Returns an iterator over the fields of the record.
    ///
    /// The iterator yields [`RecordValue`] structs
    pub fn values(self) -> RecordValueIter<'a> {
        RecordValueIter::new(self)
    }
}

macro_rules! try_into {
    ($type:ty) => {
        impl<'a> TryFrom<RecordValue<'a>> for $type {
            type Error = TableGenError;

            fn try_from(record_value: RecordValue<'a>) -> Result<Self, Self::Error> {
                Ok(record_value.init.try_into()?)
            }
        }
    };
}

try_into!(bool);
try_into!(Vec<bool>);
try_into!(Vec<BitInit<'a>>);
try_into!(i64);
try_into!(ListInit<'a>);
try_into!(DagInit<'a>);
try_into!(Record<'a>);
try_into!(String);
try_into!(&'a str);

impl<'a> From<RecordValue<'a>> for TypedInit<'a> {
    fn from(value: RecordValue<'a>) -> Self {
        value.init
    }
}

/// Struct that represents a field of a [`Record`].
///
/// Can be converted into a Rust type using the [`TryInto`] trait.
#[derive(Debug, Clone, Copy)]
pub struct RecordValue<'a> {
    pub name: StringInit<'a>,
    pub init: TypedInit<'a>,
}

impl<'a> RecordValue<'a> {
    /// Creates a record from a raw object.
    ///
    /// # Safety
    ///
    /// The raw object must be valid.
    pub unsafe fn from_raw(ptr: TableGenRecordValRef) -> Self {
        let name = StringInit::from_raw(tableGenRecordValGetNameInit(ptr));
        let value = TypedInit::from_raw(tableGenRecordValGetValue(ptr));
        Self { name, init: value }
    }
}

pub struct RecordValueIter<'a> {
    record: TableGenRecordRef,
    current: TableGenRecordValRef,
    _reference: PhantomData<&'a TableGenRecordRef>,
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
            unsafe { Some(RecordValue::from_raw(next)) }
        }
    }
}
