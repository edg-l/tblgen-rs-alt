#!/bin/sh

set -e

llvm_version=18

brew update
brew install llvm@$llvm_version

echo TABLEGEN_180_PREFIX=$(brew --prefix llvm@$llvm_version) >>$GITHUB_ENV
echo PATH=$TABLEGEN_180_PREFIX/bin:$PATH >>$GITHUB_ENV
