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

using ctablegen::RecordMap;

void tableGenRecordKeeperFree(TableGenRecordKeeperRef rk_ref) {
  delete unwrap(rk_ref);
}

TableGenRecordKeeperIteratorRef
tableGenRecordKeeperGetFirstClass(TableGenRecordKeeperRef rk_ref) {
  return wrap(new RecordMap::const_iterator(unwrap(rk_ref)->getClasses().begin()));
}

TableGenRecordKeeperIteratorRef
tableGenRecordKeeperGetFirstDef(TableGenRecordKeeperRef rk_ref) {
  return wrap(new RecordMap::const_iterator(unwrap(rk_ref)->getDefs().begin()));
}

void tableGenRecordKeeperGetNextClass(TableGenRecordKeeperIteratorRef *item) {
  auto *it = unwrap(*item);
  auto end = (*it)->second->getRecords().getClasses().end();
  if (++*it == end) {
    delete it;    
    *item = nullptr;
  }
}

void tableGenRecordKeeperGetNextDef(TableGenRecordKeeperIteratorRef *item) {
  auto *it = unwrap(*item);
  auto end = (*it)->second->getRecords().getDefs().end();
  if (++*it == end) {
    delete it;    
    *item = nullptr;
  }
}

void tableGenRecordKeeperIteratorFree(TableGenRecordKeeperIteratorRef item) {
  if (item)
    delete unwrap(item);
}

TableGenRecordKeeperIteratorRef tableGenRecordKeeperIteratorClone(TableGenRecordKeeperIteratorRef item) {
  return wrap(new RecordMap::const_iterator(*unwrap(item)));
}

const char *tableGenRecordKeeperItemGetName(TableGenRecordKeeperIteratorRef item) {
  return (*unwrap(item))->first.c_str();
}

TableGenRecordRef tableGenRecordKeeperItemGetRecord(TableGenRecordKeeperIteratorRef item) {
  return wrap((*unwrap(item))->second.get());
}

TableGenRecordMapRef
tableGenRecordKeeperGetClasses(TableGenRecordKeeperRef rk_ref) {
  return wrap(&unwrap(rk_ref)->getClasses());
}

TableGenRecordMapRef
tableGenRecordKeeperGetDefs(TableGenRecordKeeperRef rk_ref) {
  return wrap(&unwrap(rk_ref)->getDefs());
}

TableGenRecordRef tableGenRecordKeeperGetClass(TableGenRecordKeeperRef rk_ref,
                                               const char *name) {
  return wrap(unwrap(rk_ref)->getClass(std::string(name)));
}

TableGenRecordRef tableGenRecordKeeperGetDef(TableGenRecordKeeperRef rk_ref,
                                             const char *name) {
  return wrap(unwrap(rk_ref)->getDef(std::string(name)));
}

TableGenRecordVectorRef
tableGenRecordKeeperGetAllDerivedDefinitions(TableGenRecordKeeperRef rk_ref,
                                             const char *className) {
  return wrap(new ctablegen::RecordVector(
      std::move(unwrap(rk_ref)->getAllDerivedDefinitions(className))));
}

TableGenRecordRef tableGenRecordVectorGet(TableGenRecordVectorRef vec_ref,
                                          size_t index) {
  auto *vec = unwrap(vec_ref);
  if (index < vec->size())
    return wrap(((*vec)[index]));
  return nullptr;
}

void tableGenRecordVectorFree(TableGenRecordVectorRef vec_ref) {
  delete unwrap(vec_ref);
}
