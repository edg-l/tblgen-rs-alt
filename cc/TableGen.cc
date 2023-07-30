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
using ctablegen::tableGenFromRecType;

// TableGen
TableGenRef tableGenInitialize(const char *source, const size_t includes_sz,
                               const char *includes[]) {
  auto rk = new RecordKeeper;
  auto sm = new SourceMgr;

  // Check that the input table definition exists
  ErrorOr<std::unique_ptr<MemoryBuffer>> FileOrErr =
      MemoryBuffer::getMemBuffer(source);

  if (std::error_code EC = FileOrErr.getError()) {
    return nullptr;
  }

  // Add the table definition source
  sm->AddNewSourceBuffer(std::move(*FileOrErr), SMLoc());

  // Add the include directories for any table definition dependencies
  std::vector<std::string> includes_v;
  for (size_t i = 0; i < includes_sz; i++) {
    includes_v.push_back(std::string(includes[i]));
  }
  sm->setIncludeDirs(includes_v);

  return wrap(new ctablegen::TableGen(rk, sm));
}

void tableGenFree(TableGenRef tg_ref) { delete unwrap(tg_ref); }

TableGenRecordKeeperRef tableGenGetRecordKeeper(TableGenRef tg_ref) {
  return wrap(unwrap(tg_ref)->record_keeper());
}

TableGenBool tableGenParse(TableGenRef tg_ref) {
  return !unwrap(tg_ref)->Parse();
}

// LLVM ListType
TableGenRecTyKind tableGenListRecordGetType(TableGenRecordValRef rv_ref) {
  if (!rv_ref)
    return TableGenInvalidRecTyKind;
  auto rv = unwrap(rv_ref);

  if (rv->getType()->getRecTyKind() == RecTy::ListRecTyKind) {
    auto list = rv->getType()->getListTy();
    return tableGenFromRecType(list->getElementType());
  }

  return TableGenInvalidRecTyKind;
}

TableGenTypedInitRef tableGenListRecordGet(TableGenTypedInitRef rv_ref,
                                           size_t index) {
  auto list = dyn_cast<ListInit>(unwrap(rv_ref));
  if (!list)
    return nullptr;
  if (index >= list->size())
    return nullptr;
  auto elem = dyn_cast<TypedInit>(list->getElement(index));
  if (!elem)
    return nullptr;
  return wrap(elem);
}

// LLVM DagType
TableGenTypedInitRef tableGenDagRecordGet(TableGenTypedInitRef rv_ref,
                                          size_t index) {
  auto dag = dyn_cast<DagInit>(unwrap(rv_ref));
  if (!dag)
    return nullptr;
  if (index >= dag->getNumArgs())
    return nullptr;
  auto arg = dyn_cast<TypedInit>(dag->getArg(index));
  if (!arg)
    return nullptr;
  return wrap(arg);
}

size_t tableGenDagRecordNumArgs(TableGenTypedInitRef rv_ref) {
  auto dag = dyn_cast<DagInit>(unwrap(rv_ref));
  if (!dag)
    return 0;
  return dag->getNumArgs();
}

const char *tableGenDagRecordArgName(TableGenTypedInitRef rv_ref,
                                     size_t index) {
  auto dag = dyn_cast<DagInit>(unwrap(rv_ref));
  if (!dag)
    return nullptr;
  if (index >= dag->getNumArgs())
    return nullptr;
  return dag->getArgNameStr(index).data();
}

// Memory
void tableGenBitArrayFree(int8_t bit_array[]) { delete[] bit_array; }

void tableGenStringFree(const char *str) { delete str; }

void tableGenStringArrayFree(const char **str_array) { delete str_array; }
