// Original work Copyright 2016 Alexander Stocko <as@coder.gg>.
// Modified work Copyright 2023 Daan Vanoverloop
// See the COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#ifndef _CTABLEGEN_TYPES_H_
#define _CTABLEGEN_TYPES_H_

#ifdef __cplusplus
extern "C" {
#endif

typedef int TableGenBool;

typedef struct TableGen *TableGenParserRef;
typedef struct TableGenRecordKeeper *TableGenRecordKeeperRef;

typedef struct TableGenRecordMap *TableGenRecordMapRef;
typedef struct TableGenRecordMapIterator *TableGenRecordKeeperIteratorRef;
typedef struct TableGenRecordVector *TableGenRecordVectorRef;
typedef struct TableGenRecordArray *TableGenRecordArrayRef;

typedef struct TableGenRecord *TableGenRecordRef;
typedef struct TableGenRecordVal *TableGenRecordValRef;
typedef struct TableGenRecordValArray *TableGenRecordValArrayRef;

typedef struct TableGenTypedInit *TableGenTypedInitRef;

typedef struct TableGenDagPair *TableGenDagPairRef;

#ifdef __cplusplus
}
#endif

#endif
