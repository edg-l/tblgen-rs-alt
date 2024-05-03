#!/bin/sh

set -e

llvm_version=18

brew install llvm@$llvm_version

echo TABLEGEN_180_PREFIX=$(brew --prefix)/opt/llvm@$llvm_version >>$GITHUB_ENV
echo PATH=$(brew --prefix)/opt/llvm@$llvm_version/bin:$PATH >>$GITHUB_ENV
