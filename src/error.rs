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
        tableGenSourceLocationNull, TableGenDiagKind::TABLEGEN_DK_ERROR, TableGenSourceLocationRef,
    },
    string_ref::StringRef,
    util::print_callback,
    RecordValue, TableGenParser, TypedInit,
};

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum TableGenError<'a> {
    #[error("invalid TableGen source")]
    InvalidSource,
    #[error("invalid TableGen source")]
    InvalidSourceString(#[from] NulError),
    #[error("invalid UTF-8 string")]
    InvalidUtf8String(#[from] Utf8Error),
    #[error("failed to parse TableGen source")]
    Parse,
    #[error("expected field {0} in record")]
    MissingValue(String),
    #[error(transparent)]
    InitConversion(InitConversionError<'a>),
}

impl<'a> From<InitConversionError<'a>> for TableGenError<'a> {
    fn from(value: InitConversionError<'a>) -> Self {
        TableGenError::InitConversion(value)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct InitConversionError<'a> {
    init: TypedInit<'a>,
    target: &'static str,
    error: Option<String>,
}

impl<'a> InitConversionError<'a> {
    pub fn new(init: TypedInit<'a>, target: &'static str, error: Option<String>) -> Self {
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

    pub fn inner_error(&self) -> Option<&str> {
        self.error.as_ref().map(|s| s.as_str())
    }
}

impl<'a> Display for InitConversionError<'a> {
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

impl<'a> Error for InitConversionError<'a> {}

#[derive(Debug, PartialEq, Eq)]
pub struct SourceLocation<'s> {
    raw: TableGenSourceLocationRef,
    parser: &'s TableGenParser<'s>,
}

impl<'s> SourceLocation<'s> {
    pub unsafe fn from_raw(raw: TableGenSourceLocationRef, parser: &'s TableGenParser<'s>) -> Self {
        Self { raw, parser }
    }

    pub fn none(parser: &'s TableGenParser<'s>) -> Self {
        unsafe {
            Self {
                raw: tableGenSourceLocationNull(),
                parser,
            }
        }
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
