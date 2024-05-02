#!/bin/sh

set -e

llvm_version=18

brew update
brew install llvm@$llvm_version z3

echo TABLEGEN_180_PREFIX=$(brew --prefix)/opt/llvm@$llvm_version >>$GITHUB_ENV
echo PATH=$(brew --prefix)/opt/llvm@$llvm_version/bin:$PATH >>$GITHUB_ENV
echo LIBRARY_PATH=$(brew --prefix)/lib:$LIBRARY_PATH >>$GITHUB_ENV
