// Original work Copyright 2016 Alexander Stocko <as@coder.gg>.
// Modified work Copyright 2023 Daan Vanoverloop
// See the COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::{
    raw::{
        tableGenBitInitGetValue, tableGenBitsInitGetBitInit, tableGenBitsInitGetNumBits,
        tableGenDagRecordArgName, tableGenDagRecordGet, tableGenDagRecordNumArgs,
        tableGenDefInitGetValue, tableGenInitRecType, tableGenIntInitGetValue,
        tableGenListRecordGet, tableGenListRecordNumElements, tableGenStringInitGetValue,
        TableGenRecTyKind, TableGenTypedInitRef,
    },
    string_ref::StringRef,
};
use paste::paste;

use crate::{error::TableGenError, record::Record};
use std::{ffi::CStr, marker::PhantomData};

#[derive(Debug, Clone, Copy)]
pub enum TypedInit<'a> {
    Bit(BitInit<'a>),
    Bits(BitsInit<'a>),
    Code(StringInit<'a>),
    Int(IntInit<'a>),
    String(StringInit<'a>),
    List(ListInit<'a>),
    Dag(DagInit<'a>),
    Def(DefInit<'a>),
    Invalid,
}

macro_rules! as_inner {
    ($name:ident, $variant:ident, $type:ty) => {
        paste! {
            pub fn [<as_ $name>](self) -> Option<$type<'a>> {
                match self {
                    Self::$variant(v) => Some(v),
                    _ => None
                }
            }
        }
    };
}

macro_rules! try_into {
    ($variant:ident, $type:ty) => {
        impl<'a> TryFrom<TypedInit<'a>> for $type {
            type Error = TableGenError;

            fn try_from(value: TypedInit<'a>) -> Result<Self, Self::Error> {
                match value {
                    TypedInit::$variant(v) => Ok(v.try_into()?),
                    _ => Err(Self::Error::IncorrectInitType),
                }
            }
        }
    };
}

try_into!(Bit, bool);
try_into!(Bits, Vec<BitInit<'a>>);
try_into!(Bits, Vec<bool>);
try_into!(Int, i64);
try_into!(Def, Record<'a>);
try_into!(List, ListInit<'a>);
try_into!(Dag, DagInit<'a>);

impl<'a> TryFrom<TypedInit<'a>> for String {
    type Error = TableGenError;

    fn try_from(value: TypedInit<'a>) -> Result<Self, Self::Error> {
        match value {
            TypedInit::String(v) | TypedInit::Code(v) => Ok(v.try_into()?),
            _ => Err(Self::Error::IncorrectInitType),
        }
    }
}

impl<'a> TryFrom<TypedInit<'a>> for &'a str {
    type Error = TableGenError;

    fn try_from(value: TypedInit<'a>) -> Result<Self, Self::Error> {
        match value {
            TypedInit::String(v) | TypedInit::Code(v) => Ok(v.to_str()?),
            _ => Err(Self::Error::IncorrectInitType),
        }
    }
}

impl<'a> TypedInit<'a> {
    as_inner!(bit, Bit, BitInit);
    as_inner!(bits, Bits, BitsInit);
    as_inner!(code, Code, StringInit);
    as_inner!(int, Int, IntInit);
    as_inner!(string, String, StringInit);
    as_inner!(list, List, ListInit);
    as_inner!(dag, Dag, DagInit);
    as_inner!(def, Def, DefInit);

    #[allow(non_upper_case_globals)]
    pub unsafe fn from_raw(init: TableGenTypedInitRef) -> Self {
        let t = tableGenInitRecType(init);

        use TableGenRecTyKind::*;
        match t {
            TableGenBitRecTyKind => Self::Bit(BitInit::from_raw(init)),
            TableGenBitsRecTyKind => Self::Bits(BitsInit::from_raw(init)),
            TableGenDagRecTyKind => TypedInit::Dag(DagInit::from_raw(init)),
            TableGenIntRecTyKind => TypedInit::Int(IntInit::from_raw(init)),
            TableGenListRecTyKind => TypedInit::List(ListInit::from_raw(init)),
            TableGenRecordRecTyKind => Self::Def(DefInit::from_raw(init)),
            TableGenStringRecTyKind => Self::String(StringInit::from_raw(init)),
            _ => Self::Invalid,
        }
    }
}

macro_rules! init {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy)]
        pub struct $name<'a> {
            raw: TableGenTypedInitRef,
            _reference: PhantomData<&'a TableGenTypedInitRef>,
        }

        impl<'a> $name<'a> {
            pub fn from_raw(raw: TableGenTypedInitRef) -> Self {
                Self {
                    raw,
                    _reference: PhantomData,
                }
            }
        }
    };
}

init!(BitInit);

impl<'a> From<BitInit<'a>> for bool {
    fn from(value: BitInit<'a>) -> Self {
        let mut bit = -1;
        unsafe { tableGenBitInitGetValue(value.raw, &mut bit) };
        assert!(bit == 0 || bit == 1);
        bit != 0
    }
}

init!(BitsInit);

impl<'a> From<BitsInit<'a>> for Vec<BitInit<'a>> {
    fn from(value: BitsInit<'a>) -> Self {
        (0..value.num_bits())
            .map(|i| value.bit(i).expect("index within range"))
            .collect()
    }
}

impl<'a> From<BitsInit<'a>> for Vec<bool> {
    fn from(value: BitsInit<'a>) -> Self {
        (0..value.num_bits())
            .map(|i| value.bit(i).expect("index within range").into())
            .collect()
    }
}

impl<'a> BitsInit<'a> {
    pub fn bit(self, index: usize) -> Option<BitInit<'a>> {
        let bit = unsafe { tableGenBitsInitGetBitInit(self.raw, index) };
        if !bit.is_null() {
            Some(BitInit::from_raw(bit))
        } else {
            None
        }
    }

    pub fn num_bits(self) -> usize {
        let mut len = 0;
        unsafe { tableGenBitsInitGetNumBits(self.raw, &mut len) };
        len
    }
}

init!(IntInit);

impl<'a> From<IntInit<'a>> for i64 {
    fn from(value: IntInit<'a>) -> Self {
        let mut int: i64 = 0;
        let res = unsafe { tableGenIntInitGetValue(value.raw, &mut int) };
        assert!(res > 0);
        int
    }
}

init!(StringInit);

impl<'a> TryFrom<StringInit<'a>> for String {
    type Error = std::string::FromUtf8Error;

    fn try_from(value: StringInit<'a>) -> Result<Self, Self::Error> {
        String::from_utf8(value.as_bytes().to_vec())
    }
}

impl<'a> TryFrom<StringInit<'a>> for &'a str {
    type Error = std::str::Utf8Error;

    fn try_from(value: StringInit<'a>) -> Result<Self, std::str::Utf8Error> {
        value.to_str()
    }
}

impl<'a> StringInit<'a> {
    pub fn to_str(self) -> Result<&'a str, std::str::Utf8Error> {
        unsafe { StringRef::from_raw(tableGenStringInitGetValue(self.raw)) }.try_into()
    }

    pub fn as_bytes(self) -> &'a [u8] {
        unsafe { StringRef::from_raw(tableGenStringInitGetValue(self.raw)) }.into()
    }
}

init!(DefInit);

impl<'a> From<DefInit<'a>> for Record<'a> {
    fn from(value: DefInit<'a>) -> Self {
        unsafe { Record::from_raw(tableGenDefInitGetValue(value.raw)) }
    }
}

init!(DagInit);

impl<'a> DagInit<'a> {
    pub fn args(self) -> DagIter<'a> {
        DagIter {
            dag: self,
            index: 0,
        }
    }

    pub fn num_args(&self) -> usize {
        unsafe { tableGenDagRecordNumArgs(self.raw) }
    }

    pub fn name(&self, index: usize) -> Option<&'a str> {
        unsafe { StringRef::from_option_raw(tableGenDagRecordArgName(self.raw, index)) }
            .and_then(|s| s.try_into().ok())
    }

    pub fn get(&self, index: usize) -> Option<TypedInit<'a>> {
        let value = unsafe { tableGenDagRecordGet(self.raw, index) };
        if !value.is_null() {
            Some(unsafe { TypedInit::from_raw(value) })
        } else {
            None
        }
    }

    pub unsafe fn get_unchecked(&self, index: usize) -> TypedInit<'a> {
        TypedInit::from_raw(tableGenDagRecordGet(self.raw, index))
    }
}

#[derive(Debug, Clone)]
pub struct DagIter<'a> {
    dag: DagInit<'a>,
    index: usize,
}

impl<'a> Iterator for DagIter<'a> {
    type Item = (&'a str, TypedInit<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.dag.get(self.index);
        let name = self.dag.name(self.index);
        self.index += 1;
        if next.is_some() && name.is_some() {
            Some((name.unwrap(), next.unwrap()))
        } else {
            None
        }
    }
}

init!(ListInit);

impl<'a> ListInit<'a> {
    pub fn iter(self) -> ListIter<'a> {
        ListIter {
            list: self,
            index: 0,
        }
    }

    pub fn len(&self) -> usize {
        unsafe { tableGenListRecordNumElements(self.raw) }
    }

    pub fn get(&self, index: usize) -> Option<TypedInit> {
        let value = unsafe { tableGenListRecordGet(self.raw, index) };
        if !value.is_null() {
            Some(unsafe { TypedInit::from_raw(value) })
        } else {
            None
        }
    }

    pub unsafe fn get_unchecked(&self, index: usize) -> TypedInit {
        TypedInit::from_raw(tableGenListRecordGet(self.raw, index))
    }
}

#[derive(Debug, Clone)]
pub struct ListIter<'a> {
    list: ListInit<'a>,
    index: usize,
}

impl<'a> Iterator for ListIter<'a> {
    type Item = TypedInit<'a>;

    fn next(&mut self) -> Option<TypedInit<'a>> {
        let next = unsafe { tableGenListRecordGet(self.list.raw, self.index) };
        self.index += 1;
        if !next.is_null() {
            Some(unsafe { TypedInit::from_raw(next) })
        } else {
            None
        }
    }
}
