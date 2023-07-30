// Original work Copyright 2016 Alexander Stocko <as@coder.gg>.
// Modified work Copyright 2023 Daan Vanoverloop
// See the COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

pub mod error;
pub mod init;
pub mod record;
pub mod record_keeper;
mod test;

#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
pub mod raw {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use std::{
    ffi::{c_char, CString},
    sync::Mutex,
};

pub use init::TypedInit;
pub use record::Record;
pub use record::RecordValue;
pub use record_keeper::RecordKeeperRef;

use error::{Result, TableGenError};
use raw::{tableGenFree, tableGenGetRecordKeeper, tableGenInitialize, tableGenParse, TableGenRef};

// TableGen only exposes `TableGenParseFile` in its API.
// However, this function uses global state and therefore it is not thread safe.
// Until they remove this hack, we have to deal with it ourselves.
static TABLEGEN_PARSE_LOCK: Mutex<()> = Mutex::new(());

pub struct TableGen {
    raw: TableGenRef,
}

impl TableGen {
    pub fn new(source: &str, includes: &[&str]) -> Result<TableGen> {
        let source = CString::new(source).unwrap();
        let cstrings: Vec<CString> = includes.iter().map(|&i| CString::new(i).unwrap()).collect();
        let mut includes: Vec<*const c_char> = cstrings.iter().map(|i| i.as_ptr()).collect();
        let tg =
            unsafe { tableGenInitialize(source.as_ptr(), includes.len(), includes.as_mut_ptr()) };

        if tg.is_null() {
            Err(TableGenError::CreateStruct(
                "Could not initialize a TableGen instance!".into(),
            ))
        } else {
            unsafe {
                let guard = TABLEGEN_PARSE_LOCK.lock().unwrap();
                let res = if tableGenParse(tg) > 0 {
                    Ok(TableGen { raw: tg })
                } else {
                    Err(TableGenError::CreateStruct(
                        "Could not parse the source or its dependencies".into(),
                    ))
                };
                drop(guard);
                res
            }
        }
    }

    pub fn record_keeper(&self) -> RecordKeeperRef {
        unsafe { RecordKeeperRef::from_raw(tableGenGetRecordKeeper(self.raw)) }
    }
}

impl Drop for TableGen {
    fn drop(&mut self) {
        unsafe {
            tableGenFree(self.raw);
        }
    }
}
