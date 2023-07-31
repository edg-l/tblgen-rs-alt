// Original work Copyright 2016 Alexander Stocko <as@coder.gg>.
// Modified work Copyright 2023 Daan Vanoverloop
// See the COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! This crate provides raw bindings and a safe wrapper for [TableGen](https://llvm.org/docs/TableGen/),
//! a domain-specific language used by the [LLVM project](https://llvm.org/).
//!
//! The goal of this crate is to enable users to develop custom [TableGen backends](https://llvm.org/docs/TableGen/BackGuide.html)
//! in Rust. Hence the primary use case of this crate are procedural macros that
//! generate Rust code from TableGen description files.
//!
//! # Safety
//!
//! This crate aims to be completely safe.
//!
//! # Supported LLVM Versions
//!
//! An installation of LLVM is required to use this crate.
//! This crate only aims to support the latest version of LLVM. The version of
//! LLVM currently supported is 17.x.x.
//!
//! The `TABLEGEN_170_PREFIX` environment variable can be used to specify a
//! custom directory of the LLVM installation.
//!
//! # Examples
//!
//! The following example parse simple TableGen code provided as a `&str` and
//! iterates over classes and defs defined in this file.
//!
//! ```rust
//! use tblgen::{TableGenParser, RecordKeeper};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let keeper: RecordKeeper = TableGenParser::new()
//!     .add_source(
//!         r#"
//!         class A;
//!         def D: A;
//!         "#,
//!     )?
//!     .parse()?;
//! assert_eq!(keeper.classes().next().unwrap().0, Ok("A"));
//! assert_eq!(keeper.defs().next().unwrap().0, Ok("D"));
//! assert_eq!(keeper.all_derived_definitions("A").next().unwrap().name(), Ok("D"));
//! # Ok(())
//! # }
//! ```
//!
//! By adding include paths, external TableGen files can be included.
//!
//! ```rust
//! use tblgen::{TableGenParser, RecordKeeper};
//! use std::path::Path;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let keeper: RecordKeeper = TableGenParser::new()
//!     .add_source(r#"include "mlir/IR/OpBase.td""#)?
//!     .add_include_path(&Path::new(&format!("{}/include", std::env::var("TABLEGEN_170_PREFIX")?)))?
//!     .parse()?;
//! let i32_def = keeper.def("I32").expect("has I32 def");
//! assert!(i32_def.subclass_of("I"));
//! assert_eq!(i32_def.int_value("bitwidth"), Some(32));
//! # Ok(())
//! # }
//! ```
//!
//! # API Stability
//!
//! LLVM does not provide a stable C API for TableGen, and the C API provided by
//! this crate is not stable. Furthermore, the safe wrapper does not provide a
//! stable interface either, since this crate is still in early development.

/// Module containing error types.
pub mod error;
/// TableGen initialization values.
pub mod init;
/// TableGen records and record values.
pub mod record;
/// TableGen record keeper.
pub mod record_keeper;
mod string_ref;
mod test;

/// This module contains raw bindings for TableGen. Note that these bindings are
/// unstable and can change at any time.
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
pub mod raw {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use std::{ffi::CString, path::Path, sync::Mutex};

pub use init::TypedInit;
pub use record::Record;
pub use record::RecordValue;
pub use record_keeper::RecordKeeper;

use error::{Result, TableGenError};
use raw::{
    tableGenAddIncludePath, tableGenAddSource, tableGenAddSourceFile, tableGenFree, tableGenGet,
    tableGenParse, TableGenParserRef,
};

// TableGen only exposes `TableGenParseFile` in its API.
// However, this function uses global state and therefore it is not thread safe.
// Until they remove this hack, we have to deal with it ourselves.
static TABLEGEN_PARSE_LOCK: Mutex<()> = Mutex::new(());

/// Builder struct that parses TableGen source files and builds a
/// [`RecordKeeper`].
pub struct TableGenParser {
    raw: TableGenParserRef,
    source_strings: Vec<CString>,
}

impl TableGenParser {
    pub fn new() -> Self {
        Self {
            raw: unsafe { tableGenGet() },
            source_strings: Vec::new(),
        }
    }

    pub fn add_include_path(&mut self, source: &Path) -> Result<&mut Self> {
        if !source.exists() {
            return Err(TableGenError::AddInclude);
        }

        let source = CString::new(source.to_str().ok_or(TableGenError::AddSource)?)?;
        unsafe { tableGenAddIncludePath(self.raw, source.as_ptr()) }
        Ok(self)
    }

    pub fn add_source_file(&mut self, source: &Path) -> Result<&mut Self> {
        let source = CString::new(source.to_str().ok_or(TableGenError::AddSource)?)?;
        if unsafe { tableGenAddSourceFile(self.raw, source.as_ptr()) > 0 } {
            Ok(self)
        } else {
            Err(TableGenError::AddSource)
        }
    }

    pub fn add_source(&mut self, source: &str) -> Result<&mut Self> {
        let source = CString::new(source)?;
        if unsafe { tableGenAddSource(self.raw, source.as_ptr()) > 0 } {
            // Need to store source buffer until it is parsed,
            // since tableGenAddSource doesn't copy the string.
            self.source_strings.push(source);
            Ok(self)
        } else {
            Err(TableGenError::AddSource)
        }
    }

    pub fn parse(&self) -> Result<RecordKeeper> {
        unsafe {
            let guard = TABLEGEN_PARSE_LOCK.lock().unwrap();
            let keeper = tableGenParse(self.raw);
            let res = if !keeper.is_null() {
                Ok(RecordKeeper::from_raw(keeper))
            } else {
                Err(TableGenError::Parse)
            };
            drop(guard);
            res
        }
    }
}

impl Drop for TableGenParser {
    fn drop(&mut self) {
        unsafe {
            tableGenFree(self.raw);
        }
    }
}
