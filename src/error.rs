// Copyright 2023 Daan Vanoverloop
// See the COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use thiserror::Error;

use crate::init::TypedInit;

#[derive(Error, Debug)]
pub enum TableGenError {
    #[error("error parsing TableGen file")]
    Parse,
    #[error("error adding TableGen source")]
    AddSource,
    #[error("error adding TableGen include path (directory not found)")]
    AddInclude,
    #[error("pointer is null")]
    NullPointer,
    #[error("invalid bit range")]
    InvalidBitRange,
    #[error("interior null byte in string")]
    StringNulError(#[from] std::ffi::NulError),
    #[error("incorrect init type: cannot convert {0:?} to the requested type")]
    IncorrectInitType(TypedInit),
    #[error("unknown TableGen error")]
    Unknown,
}

pub type Result<T> = std::result::Result<T, TableGenError>;
