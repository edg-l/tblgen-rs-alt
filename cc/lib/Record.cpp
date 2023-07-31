// Original work Copyright 2016 Alexander Stocko <as@coder.gg>.
// Modified work Copyright 2023 Daan Vanoverloop
// See the COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#include "TableGen.hpp"
#include "Types.h"

using ctablegen::tableGenFromRecType;

TableGenRecordKeeperRef tableGenRecordGetRecords(TableGenRecordRef record_ref) {
  return wrap(&unwrap(record_ref)->getRecords());
}

const char *tableGenRecordGetName(TableGenRecordRef record_ref) {
  return unwrap(record_ref)->getName().data();
}

TableGenRecordValRef tableGenRecordGetValue(TableGenRecordRef record_ref,
                                            const char *name) {
  return wrap(unwrap(record_ref)->getValue(StringRef(name)));
}

TableGenRecTyKind tableGenRecordGetFieldType(TableGenRecordRef record_ref,
                                             const char *name) {
  auto value = unwrap(record_ref)->getValue(StringRef(name));
  if (!value)
    return TableGenInvalidRecTyKind;
  return tableGenFromRecType(value->getType());
}

TableGenRecordValRef tableGenRecordGetFirstValue(TableGenRecordRef record_ref) {
  return wrap(unwrap(record_ref)->getValues().begin());
}

TableGenRecordValRef tableGenRecordValNext(TableGenRecordRef record,
                                           TableGenRecordValRef current) {
  auto next = std::next(ArrayRef<RecordVal>::iterator(unwrap(current)));
  if (next == unwrap(record)->getValues().end()) {
    return nullptr;
  }
  return wrap(next);
}

TableGenBool tableGenRecordIsAnonymous(TableGenRecordRef record_ref) {
  return unwrap(record_ref)->isAnonymous();
}

TableGenBool tableGenRecordIsSubclassOf(TableGenRecordRef record_ref, const char *name) {
  return unwrap(record_ref)->isSubClassOf(StringRef(name));
}
