use std::marker::PhantomData;

use crate::raw::TableGenStringRef;

#[derive(Debug, Clone, Copy)]
pub struct StringRef<'a> {
    raw: TableGenStringRef,
    _reference: PhantomData<&'a TableGenStringRef>,
}

impl<'a> StringRef<'a> {
    pub unsafe fn to_raw(self) -> TableGenStringRef {
        self.raw
    }

    pub unsafe fn from_raw(raw: TableGenStringRef) -> Self {
        Self {
            raw,
            _reference: PhantomData,
        }
    }

    pub unsafe fn from_option_raw(raw: TableGenStringRef) -> Option<Self> {
        if !raw.data.is_null() {
            Some(Self::from_raw(raw))
        } else {
            None
        }
    }
}

impl<'a> From<&'a str> for StringRef<'a> {
    fn from(value: &'a str) -> Self {
        unsafe {
            StringRef::from_raw(TableGenStringRef {
                data: value.as_ptr() as *const i8,
                len: value.len(),
            })
        }
    }
}

impl<'a> TryFrom<StringRef<'a>> for &'a str {
    type Error = std::str::Utf8Error;

    fn try_from(value: StringRef<'a>) -> Result<Self, Self::Error> {
        println!("{:#?}", value);
        std::str::from_utf8(value.into())
    }
}

impl<'a> From<StringRef<'a>> for &'a [u8] {
    fn from(value: StringRef<'a>) -> Self {
        unsafe { std::slice::from_raw_parts(value.raw.data as *const u8, value.raw.len) }
    }
}
