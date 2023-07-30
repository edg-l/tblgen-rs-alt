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

using ctablegen::RecordMap;

TableGenRecordRef tableGenRecordMapGetFirst(TableGenRecordMapRef rm_ref) {
  return wrap(unwrap(rm_ref)->begin()->second.get());
}

TableGenRecordRef tableGenRecordMapGet(TableGenRecordMapRef rm_ref,
                                       const char *name) {
  auto rm = unwrap(rm_ref);
  auto val = rm->find(name);
  if (val != rm->end()) {
    return wrap(rm->find(name)->second.get());
  }
  return nullptr;
}

const char **tableGenRecordMapGetKeys(TableGenRecordMapRef rm_ref,
                                      size_t *len) {
  auto rm = unwrap(rm_ref);
  auto sz = rm->size();
  auto str_array = new const char *[sz];
  *len = sz;
  size_t idx = 0;

  for (auto &i : *rm) {
    str_array[idx++] = i.first.c_str();
  }

  return str_array;
}
