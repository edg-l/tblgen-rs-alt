pub mod error;
pub mod record;
pub mod record_keeper;
pub mod record_map;
pub mod value;

use std::ffi::{c_char, CString};

use error::{Result, TableGenError};
use record_keeper::RecordKeeper;
use tablegen_sys::{
    tableGenFree, tableGenGetRecordKeeper, tableGenInitialize, tableGenParse, TableGenRef,
};

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
                if tableGenParse(tg) > 0 {
                    Ok(TableGen { raw: tg })
                } else {
                    Err(TableGenError::CreateStruct(
                        "Could not parse the source or its dependencies".into(),
                    ))
                }
            }
        }
    }

    pub fn record_keeper(&self) -> RecordKeeper {
        unsafe { RecordKeeper::from_raw(tableGenGetRecordKeeper(self.raw)) }
    }
}

impl Drop for TableGen {
    fn drop(&mut self) {
        unsafe {
            tableGenFree(self.raw);
        }
    }
}
