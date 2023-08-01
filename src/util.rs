use std::{
    ffi::c_void,
    fmt::{self, Formatter},
};

use crate::{raw::TableGenStringRef, string_ref::StringRef};

pub(crate) unsafe extern "C" fn print_callback(string: TableGenStringRef, data: *mut c_void) {
    let (formatter, result) = &mut *(data as *mut (&mut Formatter, fmt::Result));

    if result.is_err() {
        return;
    }

    *result = (|| {
        write!(
            formatter,
            "{}",
            TryInto::<&str>::try_into(StringRef::from_raw(string)).map_err(|_| fmt::Error)?
        )
    })();
}
