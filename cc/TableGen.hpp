// Original work Copyright 2016 Alexander Stocko <as@coder.gg>.
// Modified work Copyright 2023 Daan Vanoverloop
// See the COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#ifndef _CTABLEGEN_TABLEGEN_HPP_
#define _CTABLEGEN_TABLEGEN_HPP_

#include <memory>
#include <utility>

#include <llvm/Support/CommandLine.h>
#include <llvm/Support/FileSystem.h>
#include <llvm/Support/MemoryBuffer.h>
#include <llvm/Support/ToolOutputFile.h>
#include <llvm/TableGen/Error.h>
#include <llvm/TableGen/Parser.h>
#include <llvm/TableGen/Record.h>

#include "TableGen.h"
#include "Types.h"

using namespace llvm;

namespace ctablegen {

typedef std::map<std::string, std::unique_ptr<Record>, std::less<>> RecordMap;
typedef std::vector<Record *> RecordVector;
typedef std::pair<std::string, TypedInit *> DagPair;

class TableGen {
public:
  TableGen(RecordKeeper *rk, SourceMgr *sm)
      : record_keeper_(rk), source_mgr_(sm) {}
  bool Parse() { return TableGenParseFile(*source_mgr_, *record_keeper_); }
  inline RecordKeeper *record_keeper() const { return record_keeper_.get(); }

private:
  std::unique_ptr<RecordKeeper> record_keeper_;
  std::unique_ptr<SourceMgr> source_mgr_;
};

// Utility
TableGenRecTyKind tableGenFromRecType(RecTy *rt);
} // namespace ctablegen

DEFINE_SIMPLE_CONVERSION_FUNCTIONS(ctablegen::TableGen, TableGenRef);
DEFINE_SIMPLE_CONVERSION_FUNCTIONS(RecordKeeper, TableGenRecordKeeperRef);

DEFINE_SIMPLE_CONVERSION_FUNCTIONS(ctablegen::RecordMap, TableGenRecordMapRef);
DEFINE_SIMPLE_CONVERSION_FUNCTIONS(ctablegen::RecordVector,
                                   TableGenRecordVectorRef);
DEFINE_SIMPLE_CONVERSION_FUNCTIONS(ArrayRef<Record>, TableGenRecordArrayRef);
DEFINE_SIMPLE_CONVERSION_FUNCTIONS(ArrayRef<RecordVal>,
                                   TableGenRecordValArrayRef);

DEFINE_SIMPLE_CONVERSION_FUNCTIONS(Record, TableGenRecordRef);
DEFINE_SIMPLE_CONVERSION_FUNCTIONS(RecordVal, TableGenRecordValRef);

DEFINE_SIMPLE_CONVERSION_FUNCTIONS(TypedInit, TableGenTypedInitRef);
DEFINE_SIMPLE_CONVERSION_FUNCTIONS(ctablegen::DagPair, TableGenDagPairRef);

// DEFINE_SIMPLE_CONVERSION_FUNCTIONS(ctablegen::RecordMap::const_iterator, TableGenRecordKeeperItemRef);
DEFINE_SIMPLE_CONVERSION_FUNCTIONS(ctablegen::RecordMap::const_iterator, TableGenRecordKeeperItemRef);

// inline ctablegen::RecordMap::value_type *
// unwrap(TableGenRecordKeeperItemRef P) {
//   return reinterpret_cast<ctablegen::RecordMap::value_type *>(P);
// }

// inline TableGenRecordKeeperItemRef
// wrap(const ctablegen::RecordMap::value_type *P) {
//   return reinterpret_cast<TableGenRecordKeeperItemRef>(
//       const_cast<ctablegen::RecordMap::value_type *>(&*P));
// }
// inline ctablegen::RecordMap::const_iterator *
// unwrap(TableGenRecordKeeperItemRef P) {
//   return reinterpret_cast<ctablegen::RecordMap::const_iterator *>(P);
// }

// inline TableGenRecordKeeperItemRef
// wrap(const ctablegen::RecordMap::const_iterator *P) {
//   return reinterpret_cast<TableGenRecordKeeperItemRef>(
//       const_cast<ctablegen::RecordMap::const_iterator *>(P));
// }

#endif
