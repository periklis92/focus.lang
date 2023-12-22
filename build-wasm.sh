#!/bin/bash

ROOT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
BUILD_DIR=$ROOT_DIR/build
WASM_FILE=$ROOT_DIR/target/wasm32-unknown-unknown/release/focus_lang.wasm
PLAYGROUND_LIB_DIR=$ROOT_DIR/playground/src/lib

cargo build --target wasm32-unknown-unknown --release

wasm-bindgen --target web $WASM_FILE --out-dir $BUILD_DIR
cp -r $BUILD_DIR/* $PLAYGROUND_LIB_DIR/focus-lang
