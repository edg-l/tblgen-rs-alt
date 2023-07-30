// Original work Copyright 2016 Alexander Stocko <as@coder.gg>.
// Modified work Copyright 2023 Daan Vanoverloop
// See the COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#ifndef _CTABLEGEN_TABLEGEN_H_
#define _CTABLEGEN_TABLEGEN_H_

#ifdef __cplusplus
#include <cstddef>
#include <cstdint>
#else
#include <stddef.h>
#include <stdint.h>
#endif

#include "Types.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef enum {
  TableGenBitRecTyKind,
  TableGenBitsRecTyKind,
  TableGenCodeRecTyKind,
  TableGenIntRecTyKind,
  TableGenStringRecTyKind,
  TableGenListRecTyKind,
  TableGenDagRecTyKind,
  TableGenRecordRecTyKind,
  TableGenInvalidRecTyKind
} TableGenRecTyKind;

TableGenRef tableGenInitialize(const char *source, const size_t includes_sz,
                               const char *includes[]);
void tableGenFree(TableGenRef tg_ref);
TableGenRecordKeeperRef tableGenGetRecordKeeper(TableGenRef tg_ref);

/// NOTE: TableGen currently relies on global state within a given parser
///       invocation, so this function is not thread-safe.
TableGenBool tableGenParse(TableGenRef tg_ref);

// LLVM RecordKeeper
TableGenRecordMapRef
tableGenRecordKeeperGetClasses(TableGenRecordKeeperRef rk_ref);
TableGenRecordMapRef
tableGenRecordKeeperGetDefs(TableGenRecordKeeperRef rk_ref);
TableGenRecordRef tableGenRecordKeeperGetClass(TableGenRecordKeeperRef rk_ref,
                                               const char *name);
TableGenRecordRef tableGenRecordKeeperGetDef(TableGenRecordKeeperRef rk_ref,
                                             const char *name);
TableGenRecordVectorRef
tableGenRecordKeeperGetAllDerivedDefinitions(TableGenRecordKeeperRef rk_ref,
                                             const char *className);

TableGenRecordRef tableGenRecordVectorGet(TableGenRecordVectorRef vec_ref,
                                          size_t index);
void tableGenRecordVectorFree(TableGenRecordVectorRef vec_ref);

TableGenRecordKeeperIteratorRef
tableGenRecordKeeperGetFirstClass(TableGenRecordKeeperRef rk_ref);

TableGenRecordKeeperIteratorRef
tableGenRecordKeeperGetFirstDef(TableGenRecordKeeperRef rk_ref);

void tableGenRecordKeeperGetNextClass(TableGenRecordKeeperIteratorRef *item);
void tableGenRecordKeeperGetNextDef(TableGenRecordKeeperIteratorRef *item);

const char *tableGenRecordKeeperItemGetName(TableGenRecordKeeperIteratorRef item);
TableGenRecordRef
tableGenRecordKeeperItemGetRecord(TableGenRecordKeeperIteratorRef item);
void tableGenRecordKeeperIteratorFree(TableGenRecordKeeperIteratorRef item);
TableGenRecordKeeperIteratorRef tableGenRecordKeeperIteratorClone(TableGenRecordKeeperIteratorRef item);

// LLVM Record
TableGenRecordKeeperRef tableGenRecordGetRecords(TableGenRecordRef record_ref);
const char *tableGenRecordGetName(TableGenRecordRef record_ref);
TableGenRecordValRef tableGenRecordGetValue(TableGenRecordRef record_ref,
                                            const char *name);
TableGenRecTyKind tableGenRecordGetFieldType(TableGenRecordRef record_ref,
                                             const char *name);
TableGenBool tableGenRecordIsAnonymous(TableGenRecordRef record_ref);
TableGenBool tableGenRecordIsSubclassOf(TableGenRecordRef record_ref,
                                        const char *name);

// LLVM RecordVal
const char *tableGenRecordValGetName(TableGenRecordValRef rv_ref);
TableGenRecTyKind tableGenRecordValGetType(TableGenRecordValRef rv_ref);
TableGenTypedInitRef tableGenRecordValGetValue(TableGenRecordValRef rv_ref);
void tableGenRecordValTest(TableGenRecordValRef rv_ref);
TableGenRecordValRef tableGenRecordGetFirstValue(TableGenRecordRef record_ref);
TableGenRecordValRef tableGenRecordValNext(TableGenRecordRef record,
                                           TableGenRecordValRef current);

char *tableGenRecordValGetValAsNewString(TableGenRecordValRef rv_ref);
TableGenBool tableGenRecordValGetValAsBit(TableGenRecordValRef rv_ref,
                                          int8_t *bit);
int8_t *tableGenRecordValGetValAsBits(TableGenRecordValRef rv_ref, size_t *len);
TableGenBool tableGenRecordValGetValAsInt(TableGenRecordValRef rv_ref,
                                          int64_t *integer);
TableGenRecordRef tableGenRecordValGetValAsRecord(TableGenRecordValRef rv_ref);
TableGenRecordRef
tableGenRecordValGetValAsDefRecord(TableGenRecordValRef rv_ref);

// LLVM ListType
TableGenRecTyKind tableGenListRecordGetType(TableGenRecordValRef rv_ref);
TableGenTypedInitRef tableGenListRecordGet(TableGenTypedInitRef rv_ref,
                                           size_t index);
size_t tableGenListRecordNumElements(TableGenTypedInitRef rv_ref);

// LLVM DagType
TableGenTypedInitRef tableGenDagRecordGet(TableGenTypedInitRef rv_ref,
                                          size_t index);
const char *tableGenDagRecordArgName(TableGenTypedInitRef rv_ref, size_t index);
size_t tableGenDagRecordNumArgs(TableGenTypedInitRef rv_ref);

// Utility
TableGenRecTyKind tableGenInitRecType(TableGenTypedInitRef ti);
TableGenBool tableGenBitInitGetValue(TableGenTypedInitRef ti, int8_t *bit);
int8_t *tableGenBitsInitGetValue(TableGenTypedInitRef ti, size_t *len);
TableGenBool tableGenIntInitGetValue(TableGenTypedInitRef ti, int64_t *integer);
char *tableGenStringInitGetValueNewString(TableGenTypedInitRef ti);
TableGenRecordRef tableGenDefInitGetValue(TableGenTypedInitRef ti);

// Memory
void tableGenBitArrayFree(int8_t bit_array[]);
void tableGenStringFree(const char *str);
void tableGenStringArrayFree(const char **str_array);

#ifdef __cplusplus
}
#endif
#endif
