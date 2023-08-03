// Copyright 2023 Daan Vanoverloop
// See the COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::{
    convert::Infallible,
    error::Error,
    ffi::{c_void, NulError},
    fmt::{self, Display, Formatter},
    marker::PhantomData,
    str::Utf8Error,
};

use crate::{
    raw::{
        tableGenPrintError, tableGenSourceLocationClone, tableGenSourceLocationFree,
        TableGenDiagKind::TABLEGEN_DK_ERROR, TableGenSourceLocationRef,
    },
    string_ref::StringRef,
    util::print_callback,
    RecordValue, TableGenParser, TypedInit,
};

#[derive(Debug, PartialEq, Eq)]
pub enum InvalidSourceError {
    StringNulError(NulError),
    Other,
}

impl From<NulError> for InvalidSourceError {
    fn from(value: NulError) -> Self {
        Self::StringNulError(value)
    }
}

impl Display for InvalidSourceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "invalid TableGen source")?;
        match self {
            InvalidSourceError::StringNulError(e) => write!(f, " : {}", e),
            InvalidSourceError::Other => Ok(()),
        }
    }
}

impl Error for InvalidSourceError {}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseError;

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "errors occurred while parsing TableGen source (printed to stderr)"
        )
    }
}

impl Error for ParseError {}

#[derive(Debug, PartialEq, Eq)]
pub enum MissingOrInvalidValueError<'a, E> {
    Missing(MissingValueError),
    Coversion(RecordValueConversionError<'a, E>),
}

impl<'a, E> From<MissingValueError> for MissingOrInvalidValueError<'a, E> {
    fn from(value: MissingValueError) -> Self {
        Self::Missing(value)
    }
}

impl<'a, E: Error> From<RecordValueConversionError<'a, E>> for MissingOrInvalidValueError<'a, E> {
    fn from(value: RecordValueConversionError<'a, E>) -> Self {
        Self::Coversion(value)
    }
}

impl<'a, E: Error> Display for MissingOrInvalidValueError<'a, E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            MissingOrInvalidValueError::Missing(e) => write!(f, "{e}"),
            MissingOrInvalidValueError::Coversion(e) => write!(f, "{e}"),
        }
    }
}

impl<'a, E: Error> Error for MissingOrInvalidValueError<'a, E> {}

#[derive(Debug, PartialEq, Eq)]
pub struct MissingValueError {
    name: String,
}

impl MissingValueError {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl Display for MissingValueError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "expected field {} in record", self.name)
    }
}

impl Error for MissingValueError {}

#[derive(Debug, PartialEq, Eq)]
pub struct InitConversionError<'a, E> {
    init: TypedInit<'a>,
    target: &'static str,
    error: Option<E>,
}

impl<'a, E: Error> InitConversionError<'a, E> {
    pub fn new(init: TypedInit<'a>, target: &'static str, error: Option<E>) -> Self {
        Self {
            init,
            target,
            error,
        }
    }

    pub fn init(&self) -> TypedInit<'a> {
        self.init
    }

    pub fn target_type(&self) -> &str {
        &self.target
    }

    pub fn inner_error(&self) -> Option<&E> {
        self.error.as_ref()
    }
}

impl<'a, E: Error> Display for InitConversionError<'a, E> {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        if let Some(error) = self.inner_error() {
            write!(
                formatter,
                "while converting from {:?} to {}: {}",
                self.init, self.target, error
            )
        } else {
            write!(
                formatter,
                "invalid conversion from {:?} to {}",
                self.init, self.target
            )
        }
    }
}

impl<'a, E: Error> Error for InitConversionError<'a, E> {}

#[derive(Debug, PartialEq, Eq)]
pub struct RecordValueConversionError<'a, E> {
    value: RecordValue<'a>,
    init_error: E,
}

impl<'a, E: Error> RecordValueConversionError<'a, E> {
    pub fn new(value: RecordValue<'a>, init_error: E) -> Self {
        Self { value, init_error }
    }

    pub fn value(&self) -> RecordValue<'a> {
        self.value
    }

    pub fn inner_error(&self) -> &E {
        &self.init_error
    }
}

impl<'a, E: Error> Display for RecordValueConversionError<'a, E> {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(
            formatter,
            "while converting from record value: {}",
            self.init_error,
        )
    }
}

impl<'a, E: Error> Error for RecordValueConversionError<'a, E> {}

#[derive(Debug, PartialEq, Eq)]
pub struct SourceLocation<'s> {
    raw: TableGenSourceLocationRef,
    parser: &'s TableGenParser<'s>,
}

impl<'s> SourceLocation<'s> {
    pub unsafe fn from_raw(raw: TableGenSourceLocationRef, parser: &'s TableGenParser<'s>) -> Self {
        Self { raw, parser }
    }
}

impl<'s> Clone for SourceLocation<'s> {
    fn clone(&self) -> Self {
        unsafe { Self::from_raw(tableGenSourceLocationClone(self.raw), self.parser) }
    }
}

impl<'s> Drop for SourceLocation<'s> {
    fn drop(&mut self) {
        unsafe { tableGenSourceLocationFree(self.raw) }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceError<'s, E> {
    location: SourceLocation<'s>,
    error: E,
}

impl<'s, E: Error> SourceError<'s, E> {
    pub fn new(location: SourceLocation<'s>, error: E) -> Self {
        Self { location, error }
    }

    pub fn location(&self) -> &SourceLocation {
        &self.location
    }

    pub fn map<E2>(self, map: impl FnOnce(E) -> E2) -> SourceError<'s, E2> {
        SourceError {
            location: self.location,
            error: map(self.error),
        }
    }
}

impl<'s, E: Error> Display for SourceError<'s, E> {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        let mut data = (formatter, Ok(()));

        unsafe {
            tableGenPrintError(
                self.location.parser.raw,
                self.location.raw,
                TABLEGEN_DK_ERROR,
                StringRef::from(format!("{}", self.error).as_str()).to_raw(),
                Some(print_callback),
                &mut data as *mut _ as *mut c_void,
            );
        }

        data.1
    }
}

impl<'s, E: Error> Error for SourceError<'s, E> {}

pub trait WithLocation: Error + Sized {
    /// Creates a [`SourceError`] wrapper.
    fn with_location<'s, L: SourceLoc<'s>>(self, location: L) -> SourceError<'s, Self> {
        SourceError::new(location.source_location(), self)
    }
}

impl<E> WithLocation for E where E: Error {}

pub trait SourceLoc<'s> {
    /// Returns the source location.
    fn source_location(self) -> SourceLocation<'s>;
}

impl<'s> SourceLoc<'s> for SourceLocation<'s> {
    fn source_location(self) -> SourceLocation<'s> {
        self
    }
}
