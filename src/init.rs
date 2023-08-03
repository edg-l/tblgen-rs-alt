// Original work Copyright 2016 Alexander Stocko <as@coder.gg>.
// Modified work Copyright 2023 Daan Vanoverloop
// See the COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This module contains smart pointers that reference various `Init` types in
//! TableGen.
//!
//! Init reference types can be converted to Rust types using [`Into`] and
//! [`TryInto`]. Most conversions are cheap, except for conversion to
//! [`String`].

use crate::{
    raw::{
        tableGenBitInitGetValue, tableGenBitsInitGetBitInit, tableGenBitsInitGetNumBits,
        tableGenDagRecordArgName, tableGenDagRecordGet, tableGenDagRecordNumArgs,
        tableGenDagRecordOperator, tableGenDefInitGetValue, tableGenInitPrint, tableGenInitRecType,
        tableGenIntInitGetValue, tableGenListRecordGet, tableGenListRecordNumElements,
        tableGenStringInitGetValue, TableGenRecTyKind, TableGenTypedInitRef,
    },
    string_ref::StringRef,
    util::print_callback,
};
use paste::paste;

use crate::{error::InitConversionError, record::Record, record_keeper::RecordKeeper};
use std::{
    ffi::c_void,
    fmt::{self, Debug, Display, Formatter},
    marker::PhantomData,
    str::Utf8Error,
    string::FromUtf8Error,
};

/// Enum that holds a reference to a `TypedInit`.
#[derive(Clone, Copy, PartialEq, Eq)]
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

impl<'a> Display for TypedInit<'a> {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            TypedInit::Bit(init) => write!(formatter, "{}", &init),
            TypedInit::Bits(init) => write!(formatter, "{}", &init),
            TypedInit::Code(init) => write!(formatter, "{}", &init),
            TypedInit::Int(init) => write!(formatter, "{}", &init),
            TypedInit::String(init) => write!(formatter, "{}", &init),
            TypedInit::List(init) => write!(formatter, "{}", &init),
            TypedInit::Dag(init) => write!(formatter, "{}", &init),
            TypedInit::Def(init) => write!(formatter, "{}", &init),
            TypedInit::Invalid => write!(formatter, "Invalid"),
        }
    }
}

impl<'a> Debug for TypedInit<'a> {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "TypedInit(")?;
        match self {
            TypedInit::Bit(init) => write!(formatter, "Bit({:#?})", &init),
            TypedInit::Bits(init) => write!(formatter, "Bits({:#?})", &init),
            TypedInit::Code(init) => write!(formatter, "Code({:#?})", &init),
            TypedInit::Int(init) => write!(formatter, "Int({:#?})", &init),
            TypedInit::String(init) => write!(formatter, "String({:#?})", &init),
            TypedInit::List(init) => write!(formatter, "List({:#?})", &init),
            TypedInit::Dag(init) => write!(formatter, "Dag({:#?})", &init),
            TypedInit::Def(init) => write!(formatter, "Def({:#?})", &init),
            TypedInit::Invalid => write!(formatter, "Invalid"),
        }?;
        write!(formatter, ")")
    }
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
    ($variant:ident, $init:ty, $type:ty) => {
        impl<'a> TryFrom<TypedInit<'a>> for $type {
            type Error = InitConversionError<'a, <$type as TryFrom<$init>>::Error>;

            fn try_from(value: TypedInit<'a>) -> Result<Self, Self::Error> {
                match value {
                    TypedInit::$variant(v) => Self::try_from(v).map_err(|e| {
                        InitConversionError::new(value, std::any::type_name::<$type>(), Some(e))
                    }),
                    _ => Err(InitConversionError::new(
                        value,
                        std::any::type_name::<$type>(),
                        None,
                    )),
                }
            }
        }
    };
}

try_into!(Bit, BitInit<'a>, bool);
try_into!(Bits, BitsInit<'a>, Vec<BitInit<'a>>);
try_into!(Bits, BitsInit<'a>, Vec<bool>);
try_into!(Int, IntInit<'a>, i64);
try_into!(Def, DefInit<'a>, Record<'a>);
try_into!(List, ListInit<'a>, ListInit<'a>);
try_into!(Dag, DagInit<'a>, DagInit<'a>);

impl<'a> TryFrom<TypedInit<'a>> for String {
    type Error = InitConversionError<'a, <String as TryFrom<StringInit<'a>>>::Error>;

    fn try_from(value: TypedInit<'a>) -> Result<Self, Self::Error> {
        match value {
            TypedInit::String(v) | TypedInit::Code(v) => Self::try_from(v).map_err(|e| {
                InitConversionError::new(value, std::any::type_name::<String>(), Some(e))
            }),
            _ => Err(InitConversionError::new(
                value,
                std::any::type_name::<String>(),
                None,
            )),
        }
    }
}

impl<'a> TryFrom<TypedInit<'a>> for &'a str {
    type Error = InitConversionError<'a, <&'a str as TryFrom<StringInit<'a>>>::Error>;

    fn try_from(value: TypedInit<'a>) -> Result<Self, Self::Error> {
        match value {
            TypedInit::String(v) | TypedInit::Code(v) => v.to_str().map_err(|e| {
                InitConversionError::new(value, std::any::type_name::<String>(), Some(e))
            }),
            _ => Err(InitConversionError::new(
                value,
                std::any::type_name::<String>(),
                None,
            )),
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

    /// Creates a new init from a raw object.
    ///
    /// # Safety
    ///
    /// The raw object must be valid.
    #[allow(non_upper_case_globals)]
    pub unsafe fn from_raw(init: TableGenTypedInitRef, keeper: &'a RecordKeeper<'a>) -> Self {
        let t = tableGenInitRecType(init);

        use TableGenRecTyKind::*;
        match t {
            TableGenBitRecTyKind => Self::Bit(BitInit::from_raw(init, keeper)),
            TableGenBitsRecTyKind => Self::Bits(BitsInit::from_raw(init, keeper)),
            TableGenDagRecTyKind => TypedInit::Dag(DagInit::from_raw(init, keeper)),
            TableGenIntRecTyKind => TypedInit::Int(IntInit::from_raw(init, keeper)),
            TableGenListRecTyKind => TypedInit::List(ListInit::from_raw(init, keeper)),
            TableGenRecordRecTyKind => Self::Def(DefInit::from_raw(init, keeper)),
            TableGenStringRecTyKind => Self::String(StringInit::from_raw(init, keeper)),
            _ => Self::Invalid,
        }
    }
}

macro_rules! init {
    ($name:ident) => {
        #[derive(Clone, Copy, PartialEq, Eq)]
        pub struct $name<'a> {
            raw: TableGenTypedInitRef,
            _reference: PhantomData<&'a TableGenTypedInitRef>,
            keeper: &'a RecordKeeper<'a>,
        }

        impl<'a> $name<'a> {
            /// Creates a new init from a raw object.
            ///
            /// # Safety
            ///
            /// The raw object must be valid.
            pub unsafe fn from_raw(
                raw: TableGenTypedInitRef,
                keeper: &'a RecordKeeper<'a>,
            ) -> Self {
                Self {
                    raw,
                    _reference: PhantomData,
                    keeper,
                }
            }
        }

        impl<'a> Display for $name<'a> {
            fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
                let mut data = (formatter, Ok(()));

                unsafe {
                    tableGenInitPrint(
                        self.raw,
                        Some(print_callback),
                        &mut data as *mut _ as *mut c_void,
                    );
                }

                data.1
            }
        }

        impl<'a> Debug for $name<'a> {
            fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
                write!(formatter, "{}(", stringify!($name))?;
                Display::fmt(self, formatter)?;
                write!(formatter, ")")
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
    /// Returns the bit at the given index.
    pub fn bit(self, index: usize) -> Option<BitInit<'a>> {
        let bit = unsafe { tableGenBitsInitGetBitInit(self.raw, index) };
        if !bit.is_null() {
            Some(unsafe { BitInit::from_raw(bit, self.keeper) })
        } else {
            None
        }
    }

    /// Returns the number of bits in the init.
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
    type Error = FromUtf8Error;

    fn try_from(value: StringInit<'a>) -> Result<Self, Self::Error> {
        String::from_utf8(value.as_bytes().to_vec())
    }
}

impl<'a> TryFrom<StringInit<'a>> for &'a str {
    type Error = Utf8Error;

    fn try_from(value: StringInit<'a>) -> Result<Self, Utf8Error> {
        value.to_str()
    }
}

impl<'a> StringInit<'a> {
    /// Converts the string init to a [`&str`].
    ///
    /// # Errors
    ///
    /// Returns a [`Utf8Error`] if the string init does not contain valid UTF-8.
    pub fn to_str(self) -> Result<&'a str, Utf8Error> {
        unsafe { StringRef::from_raw(tableGenStringInitGetValue(self.raw)) }.try_into()
    }

    /// Gets the string init as a slice of bytes.
    pub fn as_bytes(self) -> &'a [u8] {
        unsafe { StringRef::from_raw(tableGenStringInitGetValue(self.raw)) }.into()
    }
}

init!(DefInit);

impl<'a: 's, 's: 'a> From<DefInit<'a>> for Record<'a> {
    fn from(value: DefInit<'a>) -> Self {
        unsafe { Record::from_raw(value.keeper, tableGenDefInitGetValue(value.raw)) }
    }
}

init!(DagInit);

impl<'a: 's, 's: 'a> DagInit<'a> {
    /// Returns an iterator over the arguments of the dag.
    ///
    /// The iterator yields tuples `(&str, TypedInit)`.
    pub fn args(self) -> DagIter<'a> {
        DagIter {
            dag: self,
            index: 0,
        }
    }

    /// Returns the operator of the dag as a [`Record`].
    pub fn operator(self) -> Record<'a> {
        unsafe { Record::from_raw(self.keeper, tableGenDagRecordOperator(self.raw)) }
    }

    /// Returns the number of arguments for this dag.
    pub fn num_args(self) -> usize {
        unsafe { tableGenDagRecordNumArgs(self.raw) }
    }

    /// Returns the name of the argument at the given index.
    pub fn name(self, index: usize) -> Option<&'a str> {
        unsafe { StringRef::from_option_raw(tableGenDagRecordArgName(self.raw, index)) }
            .and_then(|s| s.try_into().ok())
    }

    /// Returns the argument at the given index.
    pub fn get(self, index: usize) -> Option<TypedInit<'a>> {
        let value = unsafe { tableGenDagRecordGet(self.raw, index) };
        if !value.is_null() {
            Some(unsafe { TypedInit::from_raw(value, self.keeper) })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct DagIter<'a> {
    dag: DagInit<'a>,
    index: usize,
}

impl<'a: 's, 's: 'a> Iterator for DagIter<'a> {
    type Item = (&'a str, TypedInit<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.dag.get(self.index);
        let name = self.dag.name(self.index);
        self.index += 1;
        if let (Some(next), Some(name)) = (next, name) {
            Some((name, next))
        } else {
            None
        }
    }
}

init!(ListInit);

impl<'a> ListInit<'a> {
    /// Returns an iterator over the elements of the list.
    ///
    /// The iterator yields values of type [`TypedInit`].
    pub fn iter(self) -> ListIter<'a> {
        ListIter {
            list: self,
            index: 0,
        }
    }

    /// Returns true if the list is empty.
    pub fn is_empty(self) -> bool {
        self.len() == 0
    }

    /// Returns the length of the list.
    pub fn len(self) -> usize {
        unsafe { tableGenListRecordNumElements(self.raw) }
    }

    /// Returns the element at the given index in the list.
    pub fn get(self, index: usize) -> Option<TypedInit<'a>> {
        let value = unsafe { tableGenListRecordGet(self.raw, index) };
        if !value.is_null() {
            Some(unsafe { TypedInit::from_raw(value, self.keeper) })
        } else {
            None
        }
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
            Some(unsafe { TypedInit::from_raw(next, self.list.keeper) })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TableGenParser;

    macro_rules! test_init {
        ($name:ident, $td_field:expr, $expected:expr) => {
            #[test]
            fn $name() {
                let rk = TableGenParser::new()
                    .add_source(&format!(
                        "
                    def A {{
                        {}
                    }}
                    ",
                        $td_field
                    ))
                    .unwrap()
                    .parse()
                    .expect("valid tablegen");
                let a = rk
                    .def("A")
                    .expect("def A exists")
                    .value("a")
                    .expect("field a exists");
                assert_eq!(a.init.try_into(), Ok($expected));
            }
        };
    }

    test_init!(bit, "bit a = 0;", false);
    test_init!(
        bits,
        "bits<4> a = { 0, 0, 1, 0 };",
        vec![false, true, false, false]
    );
    test_init!(int, "int a = 42;", 42);
    test_init!(string, "string a = \"hi\";", "hi");

    #[test]
    fn dag() {
        let rk = TableGenParser::new()
            .add_source(
                "
                def ins;
                def X {
                    int i = 4;
                }
                def Y {
                    string s = \"test\";
                }
                def A {
                    dag args = (ins X:$src1, Y:$src2);
                }
                ",
            )
            .unwrap()
            .parse()
            .expect("valid tablegen");
        let a: DagInit = rk
            .def("A")
            .expect("def A exists")
            .value("args")
            .expect("field args exists")
            .try_into()
            .expect("is dag init");
        assert_eq!(a.num_args(), 2);
        assert_eq!(a.operator().name(), Ok("ins"));
        let args = a.args();
        assert_eq!(
            args.clone().next().map(|(name, init)| (
                name,
                Record::try_from(init).expect("is record").int_value("i")
            )),
            Some(("src1", Some(4)))
        );
        assert_eq!(
            args.skip(1).next().map(|(name, init)| (
                name,
                Record::try_from(init).expect("is record").string_value("s")
            )),
            Some(("src2", Some("test".into())))
        );
    }

    #[test]
    fn list() {
        let rk = TableGenParser::new()
            .add_source(
                "
                def A {
                    list<int> l = [0, 1, 2, 3];
                }
                ",
            )
            .unwrap()
            .parse()
            .expect("valid tablegen");
        let l: ListInit = rk
            .def("A")
            .expect("def A exists")
            .value("l")
            .expect("field args exists")
            .try_into()
            .expect("is list init");
        assert_eq!(l.len(), 4);
        let iter = l.iter();
        assert_eq!(iter.clone().count(), 4);
        assert_eq!(iter.clone().nth(0).unwrap().try_into(), Ok(0));
        assert_eq!(iter.clone().nth(1).unwrap().try_into(), Ok(1));
        assert_eq!(iter.clone().nth(2).unwrap().try_into(), Ok(2));
        assert_eq!(iter.clone().nth(3).unwrap().try_into(), Ok(3));
    }
}
