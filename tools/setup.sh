#!/bin/sh

set -e

llvm_version=19

brew update
brew install llvm@$llvm_version z3

echo TABLEGEN_190_PREFIX=$(brew --prefix)/opt/llvm@$llvm_version >>$GITHUB_ENV
echo PATH=$(brew --prefix)/opt/llvm@$llvm_version/bin:$PATH >>$GITHUB_ENV
echo LIBRARY_PATH=$(brew --prefix)/lib:$LIBRARY_PATH >>$GITHUB_ENV
echo LD_LIBRARY_PATH=$(brew --prefix)/lib:$LD_LIBRARY_PATH >>$GITHUB_ENV
