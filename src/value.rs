use tablegen_sys::{
    tableGenDagItrFree, tableGenListRecordGetValues, TableGenListItr, TableGenListItrRef,
    TableGenRecordValRef,
};

use crate::record::Record;
use std::ffi::CStr;

#[derive(Debug)]
pub enum TypedValue {
    Bit(i8),
    Bits(Vec<i8>),
    Code(String),
    Int(i64),
    String(String),
    List(ListValue),
    Dag(DagValue),
    Record(Record),
    Invalid,
}

#[derive(Debug)]
pub struct DagValue {
    dag_ptr: TableGenRecordValRef,
}

impl DagValue {
    pub fn from_ptr(val: TableGenRecordValRef) -> DagValue {
        DagValue { dag_ptr: val }
    }

    // pub fn values_iter(&self) -> Result<DagIterator> {
    //     tg_ffi!(TGDagRecordGetValues, self.dag_ptr, DagIterator::from_ptr)
    // }
}

// pub struct DagIterator {
//     iter: *const CDagIterator,
// }

// impl DagIterator {
//     fn from_ptr(di: *const CDagIterator) -> DagIterator {
//         DagIterator { iter: di }
//     }
// }

// impl Iterator for DagIterator {
//     type Item = (String, TypedValue);

//     fn next(&mut self) -> Option<(String, TypedValue)> {
//         let dp_ref = unsafe { TGDagItrNextPair(self.iter) };
//         let ti: Result<TypedInit> = tg_ffi!(TGDagPairGetValue, dp_ref, TypedInit::from_ptr);

//         let dp = unsafe {
//             let name_ptr = TGDagPairGetKey(dp_ref);
//             let name = {
//                 if name_ptr.is_null() {
//                     None
//                 } else {
//                     Some(CStr::from_ptr(name_ptr).to_string_lossy().into_owned())
//                 }
//             };
//             (name, ti)
//         };

//         match dp {
//             (None, Err(_)) => None,
//             (Some(x), Err(_)) => Some((x, TypedValue::Invalid)),
//             (None, Ok(x)) => Some((String::from(""), x.to_typed_value())),
//             (Some(x), Ok(y)) => Some((x, y.to_typed_value())),
//         }
//     }
// }

// impl Drop for DagIterator {
//     fn drop(&mut self) {
//         unsafe { tableGenDagItrFree(self.iter) }
//     }
// }

#[derive(Debug)]
pub struct ListValue {
    raw: TableGenRecordValRef,
}

impl ListValue {
    pub fn from_ptr(val: TableGenRecordValRef) -> ListValue {
        ListValue { raw: val }
    }

    // pub fn values_iter(&self) -> ListIterator {
    //     ListIterator::from_raw(unsafe { tableGenListRecordGetValues(self.raw) })
    // }
}

// pub struct ListIterator {
//     raw: TableGenListItrRef,
// }

// impl ListIterator {
//     fn from_raw(di: TableGenListItrRef) -> ListIterator {
//         ListIterator { raw: di }
//     }
// }

// impl Iterator for ListIterator {
//     type Item = TypedValue;

//     fn next(&mut self) -> Option<TypedValue> {
//         let li: Result<TypedInit> = tg_ffi!(TGListItrNext, self.iter, TypedInit::from_ptr);
//         if let Ok(li) = li {
//             Some(li.to_typed_value())
//         } else {
//             None
//         }
//     }
// }

// impl Drop for ListIterator {
//     fn drop(&mut self) {
//         unsafe { TGListItrFree(self.iter) }
//     }
// }
